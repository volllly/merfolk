use merfolk::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

use wildmatch::WildMatch;

use std::{marker::PhantomData, sync::Arc};

use anyhow::Result;
use thiserror::Error;

use log::{Level, Metadata, Record};

#[derive(Debug, Error)]
pub enum Error {
  #[error("backend error: {0}")]
  FromBackend(#[from] anyhow::Error),
  #[error("could not set logger: {0}")]
  SetLogger(#[from] log::SetLoggerError),
  #[error("no sink provided in init()")]
  NoSink,
  #[error("error parsing nog level {level}")]
  LevelParseError { level: String },
}

struct LoggerInstance {
  level: Level,
  ignore_targets: Arc<Option<Vec<&'static str>>>,
  allow_targets: Arc<Option<Vec<&'static str>>>,

  #[allow(clippy::type_complexity)]
  call: Arc<dyn Fn(&Record) + 'static + Send + Sync>,
}

#[derive(derive_builder::Builder)]
#[builder(pattern = "owned")]
pub struct Logger<'a, B: Backend> {
  #[builder(setter(into, strip_option), default = "None")]
  level: Option<Level>,

  #[builder(setter(name = "sink_setter", strip_option), private, default = "None")]
  sink: Option<Box<dyn Fn(Level, String)>>,

  #[builder(setter(name = "ignore_targets_setter", strip_option), private, default = "Arc::new(None)")]
  ignore_targets: Arc<Option<Vec<&'static str>>>,

  #[builder(setter(name = "allow_targets_setter", strip_option), private, default = "Arc::new(None)")]
  allow_targets: Arc<Option<Vec<&'static str>>>,

  #[builder(private, default = "PhantomData")]
  __phantom: std::marker::PhantomData<&'a B>,
}

impl<'a, B: Backend> LoggerBuilder<'a, B> {
  pub fn sink<S: 'static + Fn(Level, String)>(self, value: S) -> Self {
    self.sink_setter(Box::new(value))
  }

  pub fn ignore_targets(self, mut value: Vec<&'static str>) -> Self {
    value.push("merfolk");

    self.ignore_targets_setter(Arc::new(Some(value)))
  }

  pub fn allow_targets(self, value: Vec<&'static str>) -> Self {
    self.allow_targets_setter(Arc::new(Some(value)))
  }
}

impl<'a, B: Backend> Logger<'a, B> {
  pub fn builder() -> LoggerBuilder<'a, B> {
    LoggerBuilder::default()
  }
}

unsafe impl<'a, T: Backend> Send for Logger<'a, T> {}

impl log::Log for LoggerInstance {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
      && !metadata.target().is_empty()
      && if let Some(ignore_targets) = self.ignore_targets.as_ref() {
        !ignore_targets.iter().any(|t| WildMatch::new(t).matches(&metadata.target()))
      } else {
        true
      }
      && if let Some(allow_targets) = self.allow_targets.as_ref() {
        allow_targets.iter().any(|t| WildMatch::new(t).matches(&metadata.target()))
      } else {
        true
      }
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      self.call.as_ref()(record);
    }
  }

  fn flush(&self) {}
}

impl<'a, B: Backend> Frontend for Logger<'a, B> {
  type Backend = B;

  fn register<T>(&mut self, caller: T) -> Result<()>
  where
    T: Fn(Call<B::Intermediate>) -> Result<Reply<B::Intermediate>> + 'static + Send + Sync,
  {
    if let Some(level) = self.level {
      log::set_boxed_logger(Box::new(LoggerInstance {
        level,
        ignore_targets: self.ignore_targets.clone(),
        allow_targets: self.allow_targets.clone(),
        call: Arc::new(move |record: &Record| {
          let args = match B::serialize(&record.args().to_string()) {
            Ok(ser) => ser,
            Err(err) => B::serialize(&err.to_string()).unwrap(),
          };

          caller(Call {
            procedure: record.level().to_string(),
            payload: args,
          })
          .ok();
        }),
      }))
      .map_err(Error::SetLogger)?;
      log::set_max_level(level.to_level_filter());
    }
    Ok(())
  }

  fn receive(&self, call: Call<B::Intermediate>) -> Result<Reply<B::Intermediate>> {
    self.sink.as_ref().ok_or(Error::NoSink)?(
      match call.procedure.as_str() {
        "ERROR" => Level::Error,
        "WARN" => Level::Warn,
        "INFO" => Level::Info,
        "DEBUG" => Level::Debug,
        "TRACE" => Level::Trace,
        err => return Err(Error::LevelParseError { level: err.to_string() }.into()),
      },
      B::deserialize(&call.payload)?,
    );

    Ok(Reply { payload: B::serialize(&())? })
  }
}

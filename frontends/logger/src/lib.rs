use mer::{
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

pub struct Logger<'a, B: Backend> {
  level: Option<Level>,
  sink: Option<Box<dyn Fn(Level, String)>>,
  ignore_targets: Arc<Option<Vec<&'static str>>>,
  allow_targets: Arc<Option<Vec<&'static str>>>,

  __phantom: std::marker::PhantomData<&'a B>,
}

unsafe impl<'a, T: Backend> Send for Logger<'a, T> {}

pub struct LoggerInit {
  pub level: Option<Level>,
  pub sink: Option<Box<dyn Fn(Level, String)>>,
  pub ignore_targets: Option<Vec<&'static str>>,
  pub allow_targets: Option<Vec<&'static str>>,
}

impl Default for LoggerInit {
  fn default() -> Self {
    LoggerInit {
      level: None,
      sink: None,
      ignore_targets: None,
      allow_targets: None,
    }
  }
}

impl LoggerInit {
  pub fn init<'a, B: Backend>(self) -> Logger<'a, B> {
    let mut ignore_targets = self.ignore_targets.unwrap_or_default();

    ignore_targets.push("mer");

    Logger {
      level: self.level,
      sink: self.sink,
      ignore_targets: Arc::new(ignore_targets.into()),
      allow_targets: Arc::new(self.allow_targets),

      __phantom: PhantomData,
    }
  }
}

impl log::Log for LoggerInstance {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
      && !metadata.target().is_empty()
      && if let Some(ignore_targets) = self.ignore_targets.as_ref() {
        !ignore_targets.iter().any(|t| WildMatch::new(t).is_match(&metadata.target()))
      } else {
        true
      }
      && if let Some(allow_targets) = self.allow_targets.as_ref() {
        allow_targets.iter().any(|t| WildMatch::new(t).is_match(&metadata.target()))
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

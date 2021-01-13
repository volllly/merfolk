use mer::{
  interfaces::{Backend, Frontend},
  Call, Reply,
};

use std::{marker::PhantomData, sync::Arc};

use snafu::{OptionExt, ResultExt, Snafu};

use log::{Level, Metadata, Record};

#[derive(Debug, Snafu)]
pub enum Error<B: core::fmt::Display> {
  FromBackend { from: B },
  SetLogger { source: log::SetLoggerError },
  NoSink,
  WriteSink,
  LevelParseError { level: String },
}

impl<B: snafu::Error> From<B> for Error<B> {
  fn from(from: B) -> Self {
    Error::FromBackend { from }
  }
}

struct LoggerInstance {
  level: Level,
  ignore_crates: Arc<Vec<&'static str>>,

  #[allow(clippy::type_complexity)]
  call: Arc<dyn Fn(&Record) + 'static + Send + Sync>,
}

pub struct Logger<'a, B: Backend<'a>> {
  level: Option<Level>,
  sink: Option<Box<dyn Fn(Level, String)>>,
  ignore_crates: Arc<Vec<&'static str>>,

  __phantom: std::marker::PhantomData<&'a B>,
}

unsafe impl<'a, T: Backend<'a>> Send for Logger<'a, T> {}

pub struct LoggerInit {
  pub level: Option<Level>,
  pub sink: Option<Box<dyn Fn(Level, String)>>,
  pub ignore_crates: Vec<&'static str>,
}

impl Default for LoggerInit {
  fn default() -> Self {
    LoggerInit {
      level: None,
      sink: None,
      ignore_crates: vec![],
    }
  }
}

impl LoggerInit {
  pub fn init<'a, B: Backend<'a>>(mut self) -> Logger<'a, B> {
    self.ignore_crates.push("mer");

    Logger {
      level: self.level,
      sink: self.sink,
      ignore_crates: Arc::new(self.ignore_crates),

      __phantom: PhantomData,
    }
  }
}

impl log::Log for LoggerInstance {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level && !self.ignore_crates.contains(&metadata.target())
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      self.call.as_ref()(record);
    }
  }

  fn flush(&self) {}
}

impl<'a, B> Frontend<'a, B> for Logger<'a, B>
where
  B: Backend<'a>,
{
  type Intermediate = String;
  type Error = Error<B::Error>;

  fn caller<T>(&mut self, caller: T) -> Result<(), Self::Error>
  where
    T: Fn(&Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>, B::Error> + 'static + Send + Sync,
  {
    if let Some(level) = self.level {
      log::set_boxed_logger(Box::new(LoggerInstance {
        level,
        ignore_crates: self.ignore_crates.clone(),
        call: Arc::new(move |record: &Record| {
          let args = match B::serialize(&record.args().to_string()) {
            Ok(ser) => ser,
            Err(err) => B::serialize(&err.to_string()).unwrap(),
          };

          caller(&Call {
            procedure: record.level().to_string(),
            payload: &args,
          })
          .ok();
        }),
      }))
      .context(SetLogger {})?;
      log::set_max_level(level.to_level_filter());
    }
    Ok(())
  }

  fn receive(&self, call: &Call<&B::Intermediate>) -> Result<Reply<B::Intermediate>, Error<B::Error>> {
    self.sink.as_ref().context(NoSink)?(
      match call.procedure.as_str() {
        "ERROR" => Level::Error,
        "WARN" => Level::Warn,
        "INFO" => Level::Info,
        "DEBUG" => Level::Debug,
        "TRACE" => Level::Trace,
        err => return Err(Error::LevelParseError { level: err.to_string() }),
      },
      B::deserialize(&call.payload)?,
    );

    Ok(Reply { payload: B::serialize(&())? })
  }
}

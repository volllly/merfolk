#[cfg(unix)]
use mer::*;

#[cfg(unix)]
fn add(a: i32, b: i32) -> i32 {
  a + b
}

struct MockTTY {
  pub m: Box<dyn serialport::SerialPort>,
  pub s: Box<dyn serialport::SerialPort>,
}

impl serialport::SerialPort for MockTTY {
  fn name(&self) -> Option<String> {
    Some(format!(
      "{:?} -> {:?}",
      self.m.name().unwrap_or_else(|| "_".to_string()),
      self.s.name().unwrap_or_else(|| "_".to_string())
    ))
  }

  fn baud_rate(&self) -> serialport::Result<u32> {
    self.m.baud_rate()
  }

  fn data_bits(&self) -> serialport::Result<serialport::DataBits> {
    self.m.data_bits()
  }

  fn flow_control(&self) -> serialport::Result<serialport::FlowControl> {
    self.m.flow_control()
  }

  fn parity(&self) -> serialport::Result<serialport::Parity> {
    self.m.parity()
  }

  fn stop_bits(&self) -> serialport::Result<serialport::StopBits> {
    self.m.stop_bits()
  }

  fn timeout(&self) -> std::time::Duration {
    self.m.timeout()
  }

  fn set_baud_rate(&mut self, baud_rate: u32) -> serialport::Result<()> {
    self.m.set_baud_rate(baud_rate)?;
    self.s.set_baud_rate(baud_rate)?;
    Ok(())
  }

  fn set_data_bits(&mut self, data_bits: serialport::DataBits) -> serialport::Result<()> {
    self.m.set_data_bits(data_bits)?;
    self.s.set_data_bits(data_bits)?;
    Ok(())
  }

  fn set_flow_control(&mut self, flow_control: serialport::FlowControl) -> serialport::Result<()> {
    self.m.set_flow_control(flow_control)?;
    self.s.set_flow_control(flow_control)?;
    Ok(())
  }

  fn set_parity(&mut self, parity: serialport::Parity) -> serialport::Result<()> {
    self.m.set_parity(parity)?;
    self.s.set_parity(parity)?;
    Ok(())
  }

  fn set_stop_bits(&mut self, stop_bits: serialport::StopBits) -> serialport::Result<()> {
    self.m.set_stop_bits(stop_bits)?;
    self.s.set_stop_bits(stop_bits)?;
    Ok(())
  }

  fn set_timeout(&mut self, timeout: std::time::Duration) -> serialport::Result<()> {
    self.m.set_timeout(timeout)?;
    self.s.set_timeout(timeout)?;
    Ok(())
  }

  fn write_request_to_send(&mut self, level: bool) -> serialport::Result<()> {
    self.s.write_request_to_send(level)
  }

  fn write_data_terminal_ready(&mut self, level: bool) -> serialport::Result<()> {
    self.s.write_data_terminal_ready(level)
  }

  fn read_clear_to_send(&mut self) -> serialport::Result<bool> {
    self.m.read_clear_to_send()
  }

  fn read_data_set_ready(&mut self) -> serialport::Result<bool> {
    self.m.read_data_set_ready()
  }

  fn read_ring_indicator(&mut self) -> serialport::Result<bool> {
    self.m.read_ring_indicator()
  }

  fn read_carrier_detect(&mut self) -> serialport::Result<bool> {
    self.m.read_carrier_detect()
  }

  fn bytes_to_read(&self) -> serialport::Result<u32> {
    self.s.bytes_to_read()
  }

  fn bytes_to_write(&self) -> serialport::Result<u32> {
    self.m.bytes_to_write()
  }

  fn clear(&self, buffer_to_clear: serialport::ClearBuffer) -> serialport::Result<()> {
    self.m.clear(buffer_to_clear)?;
    self.s.clear(buffer_to_clear)?;
    Ok(())
  }

  fn try_clone(&self) -> serialport::Result<Box<dyn serialport::SerialPort>> {
    Err(serialport::Error::new(serialport::ErrorKind::Unknown, "cannot clone"))
  }

  fn set_break(&self) -> serialport::Result<()> {
    self.m.set_break()
  }

  fn clear_break(&self) -> serialport::Result<()> {
    self.m.clear_break()
  }
}

impl std::io::Write for MockTTY {
  fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
    self.m.write(buf)
  }

  fn flush(&mut self) -> std::io::Result<()> {
    self.m.flush()
  }
}

impl std::io::Read for MockTTY {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    self.s.read(buf)
  }
}

#[test]
#[cfg(unix)]
fn register_serialport() {
  let register_caller = mer_frontend_register::RegisterInit {}.init();
  let register_receiver = mer_frontend_register::RegisterInit {}.init();
  register_caller.register("add", |(a, b)| add(a, b)).unwrap();
  register_receiver.register("add", |(a, b)| add(a, b)).unwrap();

  let pairs = (serialport::TTYPort::pair().unwrap(), serialport::TTYPort::pair().unwrap());

  let port_caller = MockTTY {
    m: Box::new(pairs.0 .0),
    s: Box::new(pairs.1 .1),
  };

  let port_receiver = MockTTY {
    m: Box::new(pairs.1 .0),
    s: Box::new(pairs.0 .1),
  };

  let mut mer_caller = MerInit {
    backend: mer_backend_serialport::SerialPortInit { port: Box::new(port_caller) }.init().unwrap(),
    frontend: register_caller,
    middlewares: None,
  }
  .init()
  .unwrap();

  let mut mer_receiver = MerInit {
    backend: mer_backend_serialport::SerialPortInit { port: Box::new(port_receiver) }.init().unwrap(),
    frontend: register_receiver,
    middlewares: None,
  }
  .init()
  .unwrap();

  mer_caller.start().unwrap();
  mer_receiver.start().unwrap();

  let (a, b) = (rand::random::<i32>() / 2, rand::random::<i32>() / 2);
  let result: i32 = mer_caller.frontend(|f| f.call("add", &(a, b)).unwrap()).unwrap();
  assert_eq!(result, a + b);
}

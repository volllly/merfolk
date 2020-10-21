use crate::interfaces::frontend;

#[derive(Debug)]
pub struct Empty {}

impl Default for Empty {
  fn default() -> Self {
    Empty {}
  }
}

impl Empty {
  pub fn new() -> Empty {
    Empty::default()
  }
}

impl<'a> frontend::Frontend<'a> for Empty {
  type Intermediate = ();

  fn receive(
    &self,
    call: &crate::Call<&Self::Intermediate>,
  ) -> Result<crate::Reply<Box<dyn erased_serde::Serialize>>, frontend::Error> {
    println!(
      "{}: {}",
      call.procedure,
      serde_json::to_string(call.payload).unwrap()
    );
    Ok(crate::Reply {
      payload: Box::new(()),
    })
  }

  fn serialize(&self, from: &dyn erased_serde::Serialize) -> &() {
    println!("{}", serde_json::to_string(from).unwrap());
    &()
  }
}

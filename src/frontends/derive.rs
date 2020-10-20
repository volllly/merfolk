use crate::interfaces::frontend;

#[derive(Debug)]
pub struct Empty {

}

impl frontend::Procedure<'_> for Empty {
    type Intermediate = ();

    #[allow(unused_variables)]
    fn call(&self, payload: &Self::Intermediate) -> Result<Self::Intermediate, frontend::Error> {
        Ok(())
    }
}

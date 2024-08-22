pub struct DrawContext {}

pub trait Vis {
    type Input;
    type Output;

    fn process_and_visualize(
        &mut self,
        input: Self::Input,
        context: &mut DrawContext,
    ) -> Self::Output;
}


// impls


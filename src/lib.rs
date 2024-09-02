use dsp::Block;
use vis::{DrawContext, VisualizeResult};

pub mod control;
pub mod dsp;
pub mod setups;
pub mod vis;

#[no_mangle]
pub extern "C" fn get_setup(ctx: &mut DrawContext) -> Box<VisualizeResult> {

    println!("?xxdasdsax");
    let mut system = setups::playground::create_playground_blocks().unwrap();
    println!("?xxdasdsax");
    let f = Box::new(system.process_and_visualize((), ctx).1);
    println!("?a");

    f
}

use anyhow::Result;

#[derive(Debug)]
pub struct Flow {
    pub enter: f32,
    pub exit: f32,
}

fn convert_volt_to_flow(adc_value: u16) -> f32 {
    // TODO calculate temperature
    return 0.0;
}

pub fn read_flow() -> Result<Flow> {
    // TODO read both flow sensors.
    return Ok(Flow {
        enter: 0.0,
        exit: 0.0,
    });
}

pub fn calculate_espresso_flow() -> Result<f32> {
    // TODO calculate the espresso flow rate in the grouphead with the flow in and out.
    let current_flow = read_flow()?;

    return Ok(current_flow.enter - current_flow.exit);
}

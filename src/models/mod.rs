pub mod ddpg;
pub mod policy_gradient;

use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum ModelType {
    Ddpg,
    PolicyGradient,
}



impl FromStr for ModelType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ddpg" => Ok(ModelType::Ddpg),
            "pg" => Ok(ModelType::PolicyGradient),
            _ => Err("no model match"),
        }
    }
}

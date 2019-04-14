use crate::broker::Broker;
use crate::config::Config;

/// Main structure for holding a Bfg
pub struct Bfg<B>
where
    B: Broker,
{
    broker: B,
    pub config: Config,
}

impl<B> Bfg<B>
where
    B: Broker,
{
    pub fn new(config: Config, broker: B) -> Bfg<B> {
        Bfg { broker, config }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

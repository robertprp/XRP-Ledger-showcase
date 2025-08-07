use xrpl::models::Amount;

pub trait AmountExt {
    fn xrp_amount(value: impl Into<String>) -> Self;
}

impl AmountExt for Amount<'_> {
    fn xrp_amount(value: impl Into<String>) -> Self {
        let value_string = value.into();
        Amount::XRPAmount(xrpl::models::XRPAmount(value_string.into()))
    }
}

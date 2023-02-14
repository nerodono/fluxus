pub struct RouterData {}

#[macro_export]
macro_rules! route {
    (
        $enum:path => [
            $(
                $variant:ident => $fn:expr
            ),*
            $(,)?
        ]($in:expr, $router_data:expr)
        else
            $fail_fn:expr
    ) => {
        match $in {
            $(
                <$enum>::$variant => $fn
            ),* ,
            _ => $fail_fn
        }
    };
}

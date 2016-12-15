

#[macro_export]
macro_rules! tap {
    ( $x:expr => $y:expr )  => {
        {
            let result = $x;
            $y;
            result
        }
    }
}

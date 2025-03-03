pub mod drawing;
pub mod functions;
pub mod protocol;

macro_rules! declare {
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, [$($code:expr),+ $(,)?]) => {
        declare!($name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?, |self| [$($code),+]);
    };
    ($name:ident $(<$($lt:tt$(:$clt:tt$(+$dlt:tt)*)?),+>)? $(($($vis:vis $ty:ty),*))?, |$self:ident| [$($code:expr),+ $(,)?]) => {
        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        pub struct $name $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $(($($vis $ty),*))?;

        impl $(<$($lt$(:$clt$(+$dlt)*)?),+>)? $crate::terminal::ToTerminal for $name $(<$($lt),+>)? {
            #[inline(always)]
            fn to_terminal(&$self, term: &mut dyn $crate::terminal::WriteableTerminal) -> std::result::Result<(), $crate::Error> {
                $($crate::terminal::ToTerminal::to_terminal(&$code, term)?;)+
                std::result::Result::Ok(())
            }
        }
    };
}

pub(crate) use declare;

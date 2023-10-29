use inator::*;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Stack {}

impl ToSrc for Stack {
    #[inline]
    fn to_src(&self) -> String {
        todo!()
    }
    #[inline]
    fn src_type() -> String {
        "types::Stack".to_owned()
    }
}

#[derive(Debug)]
pub struct Arena(pub(in crate::parse) bumpalo::Bump);

impl Arena {
	pub fn new() -> Self {
		Self(bumpalo::Bump::new())
	}
}

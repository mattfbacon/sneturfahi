#[derive(Debug, Default)]
pub struct Arena(pub(in crate::parse) bumpalo::Bump);

impl Arena {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn allocated_bytes(&self) -> usize {
		self.0.allocated_bytes()
	}
}

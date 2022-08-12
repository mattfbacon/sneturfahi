use std::marker::PhantomData;

pub type Location = u32;

#[derive(Debug, Clone, Copy)]
pub struct Span<'a> {
	pub start: Location,
	pub end: Location,
	belongs_to: PhantomData<&'a str>,
}

impl<'a> Span<'a> {
	pub fn from_slice(slice: &'a str) -> Self {
		Self {
			start: 0,
			end: Location::try_from(slice.len()).unwrap(),
			belongs_to: PhantomData,
		}
	}

	pub fn from_embedded_slice(outer_start_ptr: *const u8, embedded: &'a str) -> Self {
		let inner_start_ptr = embedded.as_ptr();
		assert!(inner_start_ptr >= outer_start_ptr);
		let start = Location::try_from(inner_start_ptr as usize - outer_start_ptr as usize).unwrap();
		let end = start + u32::try_from(embedded.len()).unwrap();
		Self {
			start,
			end,
			belongs_to: PhantomData,
		}
	}

	pub fn slice(self, text: &'a str) -> Option<&'a str> {
		self.slice_arbitrary(text)
	}

	/// Slice data that may not belong to this span.
	pub fn slice_arbitrary(self, text: &str) -> Option<&str> {
		text.get(usize::try_from(self.start).unwrap()..usize::try_from(self.end).unwrap())
	}
}

use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;

pub type Location = u32;

#[derive(Clone, Copy)]
pub struct Span<'a> {
	start: Location,
	end: Location,
	belongs_to: PhantomData<&'a str>,
}

// XXX: use `#[debug(skip)]` if/when it lands
impl Debug for Span<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.debug_struct("Span")
			.field("start", &self.start)
			.field("end", &self.end)
			.finish()
	}
}

impl<'a> Span<'a> {
	pub fn new(_input: &'a str, start: Location, end: Location) -> Self {
		let ret = Self {
			start,
			end,
			belongs_to: PhantomData,
		};
		ret.assert_valid();
		ret
	}

	#[inline(always)]
	pub fn start(self) -> Location {
		self.start
	}

	#[inline(always)]
	pub fn end(self) -> Location {
		self.end
	}

	#[inline]
	pub fn len(self) -> u32 {
		self.end - self.start
	}

	pub fn between(before: Self, after: Self) -> Self {
		let ret = Self {
			start: before.end,
			end: after.start,
			belongs_to: PhantomData,
		};
		ret.assert_valid();
		ret
	}

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

	/// Get the part of the input that is after the end of this span
	pub fn slice_after(self, text: &'a str) -> Option<&'a str> {
		self.slice_after_arbitrary(text)
	}

	/// The same as `slice_after`, but works on data that may not belong to this span
	pub fn slice_after_arbitrary(self, text: &str) -> Option<&str> {
		text.get(usize::try_from(self.end).unwrap()..)
	}

	fn assert_valid(self) {
		assert!(self.start < self.end);
	}
}

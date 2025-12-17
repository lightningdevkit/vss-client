use crate::types::KeyValue;
use core::fmt;

/// Contains [`StorableBuilder`] utility.
///
/// [`StorableBuilder`]: storable_builder::StorableBuilder
pub mod storable_builder;

/// Contains retry utilities.
pub mod retry;

/// Contains [`KeyObfuscator`] utility.
///
/// [`KeyObfuscator`]: key_obfuscator::KeyObfuscator
pub mod key_obfuscator;

pub(crate) struct KeyValueVecKeyPrinter<'a>(pub(crate) &'a Vec<KeyValue>);

impl fmt::Display for KeyValueVecKeyPrinter<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[")?;
		for (i, k) in self.0.iter().enumerate() {
			if i == self.0.len() - 1 {
				write!(f, "{}", &k.key)?;
			} else {
				write!(f, "{}, ", &k.key)?;
			}
		}
		write!(f, "]")?;
		Ok(())
	}
}

//! Certify procedure byte slots using this Memory's admitted opaque code ledger.
use super::*;

impl Memory {
    /// Called after canonical retokenization, never on an unowned numeric token.
    pub(crate) fn certify_code_image(
        &self,
        types: &dyn TypeView,
        image: &mut ByteImage,
    ) -> Result<(), Error> {
        if image.target() != self.target {
            return Err(Error::InvalidIr(
                "code image target differs from Memory target",
            ));
        }
        image.certify_code_handles(|value| {
            let receipt = self.code_pointer(types, value)?;
            if self.code_pointer_value(types, receipt, receipt.signature())? != *value {
                return Err(Error::InvalidIr(
                    "code ledger receipt changed callable identity",
                ));
            }
            Ok(Pointer::from_code(receipt, receipt.signature()))
        })
    }
}

#[cfg(test)]
mod tests;

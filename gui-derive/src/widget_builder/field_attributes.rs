use syn::Field;

#[derive(Clone)]
pub struct FieldAttributes;


impl FieldAttributes {
    pub fn new(field: Field) -> syn::Result<Self> {
        Ok(FieldAttributes)
    }
}
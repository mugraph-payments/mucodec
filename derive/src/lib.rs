use myn::prelude::*;
use proc_macro::TokenStream;

#[proc_macro_derive(Encode)]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let mut input = input.into_token_iter();

    // Parse struct attributes and visibility
    let _attrs = match input.parse_attributes() {
        Ok(attrs) => attrs,
        Err(err) => return err,
    };

    if let Err(err) = input.parse_visibility() {
        return err;
    }

    // Expect 'struct' keyword
    if let Err(err) = input.expect_ident("struct") {
        return err;
    }

    // Get struct name
    let name = match input.try_ident() {
        Ok(ident) => ident,
        Err(err) => return err,
    };

    // Parse struct fields in braces
    let fields = match input.expect_group(Delimiter::Brace) {
        Ok(mut group) => {
            let mut fields = Vec::new();
            while group.peek().is_some() {
                // Parse field visibility
                if let Err(err) = group.parse_visibility() {
                    return err;
                }

                // Get field name
                let field_name = match group.try_ident() {
                    Ok(ident) => ident,
                    Err(err) => return err,
                };

                // Expect colon after field name
                if let Err(err) = group.expect_punct(':') {
                    return err;
                }

                // Get field type
                let (field_type, _span) = match group.parse_path() {
                    Ok(ty) => ty,
                    Err(err) => return err,
                };

                fields.push((field_name, field_type));

                // Handle optional comma
                if group.peek().is_some() {
                    if let Err(err) = group.expect_punct(',') {
                        return err;
                    }
                }
            }
            fields
        }
        Err(err) => return err,
    };

    // Generate implementation
    let mut output = String::new();
    output.push_str(&format!(
        "impl ReprBytes<{{std::mem::size_of::<{}>()}}> for {} {{\n",
        name, name
    ));
    output.push_str("    fn from_bytes(input: [u8; Self::SIZE]) -> Self {\n");
    output.push_str("        let mut offset = 0;\n");
    output.push_str("        Self {\n");

    for (field_name, field_type) in &fields {
        output.push_str(&format!(
            "            {}: {{
                let size = std::mem::size_of::<{}>();
                let mut bytes = [0u8; std::mem::size_of::<{}>()];
                bytes.copy_from_slice(&input[offset..offset + size]);
                offset += size;
                <{}>::from_bytes(bytes)
            }},\n",
            field_name, field_type, field_type, field_type
        ));
    }

    output.push_str("        }\n");
    output.push_str("    }\n\n");

    output.push_str("    fn as_bytes(&self) -> [u8; Self::SIZE] {\n");
    output.push_str("        let mut result = [0u8; Self::SIZE];\n");
    output.push_str("        let mut offset = 0;\n\n");

    for (field_name, field_type) in fields {
        output.push_str(&format!(
            "        {{
            let bytes = self.{}.as_bytes();
            let size = std::mem::size_of::<{}>();
            result[offset..offset + size].copy_from_slice(&bytes);
            offset += size;
        }}\n",
            field_name, field_type
        ));
    }

    output.push_str("        result\n");
    output.push_str("    }\n");
    output.push_str("}\n");

    output.parse().unwrap()
}

use myn::prelude::*;
use proc_macro::{Delimiter, TokenStream};

#[proc_macro_derive(ReprBytes)]
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

    // Check next token to determine struct type
    match input.peek() {
        // Tuple struct with parentheses
        Some(proc_macro::TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis => {
            let group_stream = group.stream();
            let mut group = group_stream.into_token_iter();
            input.next(); // Consume the group token

            // Parse the full type including any generic parameters
            let mut field_type = String::new();
            while let Some(token) = group.next() {
                field_type.push_str(&token.to_string());
            }

            // Generate implementation for tuple struct
            let mut output = String::new();
            output.push_str(&format!("impl ::mucodec::ReprBytes<32> for {} {{\n", name));
            output.push_str("    fn from_bytes(input: [u8; 32]) -> Self {\n");
            output.push_str(&format!(
                "        Self(<{}>::from_bytes(input))\n",
                field_type
            ));
            output.push_str("    }\n\n");
            output.push_str("    fn as_bytes(&self) -> [u8; 32] {\n");
            output.push_str("        self.0.as_bytes()\n");
            output.push_str("    }\n");
            output.push_str("}\n");

            return output.parse().unwrap();
        }
        // Unit struct with semicolon
        Some(token) if token.to_string() == ";" => {
            let mut output = String::new();
            output.push_str(&format!("impl ::mucodec::ReprBytes<0> for {} {{\n", name));
            output.push_str("    fn from_bytes(_input: [u8; 0]) -> Self {\n");
            output.push_str("        Self\n");
            output.push_str("    }\n\n");
            output.push_str("    fn as_bytes(&self) -> [u8; 0] {\n");
            output.push_str("        []\n");
            output.push_str("    }\n");
            output.push_str("}\n");

            return output.parse().unwrap();
        }
        // Empty struct with braces or normal struct with fields
        Some(proc_macro::TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            let group_stream = group.stream();
            let mut group = group_stream.into_token_iter();
            input.next(); // Consume the group token after we've captured its contents

            // If group is empty, generate implementation for empty struct
            if group.peek().is_none() {
                let mut output = String::new();
                output.push_str(&format!("impl ::mucodec::ReprBytes<0> for {} {{\n", name));
                output.push_str("    fn from_bytes(_input: [u8; 0]) -> Self {\n");
                output.push_str("        Self {}\n");
                output.push_str("    }\n\n");
                output.push_str("    fn as_bytes(&self) -> [u8; 0] {\n");
                output.push_str("        []\n");
                output.push_str("    }\n");
                output.push_str("}\n");

                return output.parse().unwrap();
            }

            let mut fields = Vec::new();
            let mut group = group;
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

                // Get field type including generic parameters
                let mut field_type = String::new();
                while let Some(token) = group.next() {
                    match token {
                        proc_macro::TokenTree::Punct(p) if p.as_char() == ',' => break,
                        _ => field_type.push_str(&token.to_string()),
                    }
                }
                field_type = field_type.trim().to_string();

                fields.push((field_name, field_type));

                // Handle optional comma
                if group.peek().is_some() {
                    if let Err(err) = group.expect_punct(',') {
                        return err;
                    }
                }
            }

            // Generate implementation for struct with fields
            let mut output = String::new();

            output.push_str(&format!("impl ::mucodec::ReprBytes<32> for {} {{\n", name));
            output.push_str("    fn from_bytes(input: [u8; 32]) -> Self {\n");
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

            output.push_str("    fn as_bytes(&self) -> [u8; 32] {\n");
            output.push_str("        let mut result = [0u8; 32];\n");
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

            return output.parse().unwrap();
        }
        _ => {
            return "compile_error!(\"Expected struct definition\")"
                .parse()
                .unwrap()
        }
    }
}

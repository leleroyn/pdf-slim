pub mod info;
pub mod report;

use lopdf::Document;

use crate::analyze::report::SignatureDetails;

/// Check if a PDF has digital signatures.
/// Returns signature details if found, None if clean.
pub fn check_digital_signature(doc: &Document) -> Option<SignatureDetails> {
    check_acroform_signatures(doc)
        .or_else(|| check_docmdp(doc))
        .or_else(|| check_permissions_dict(doc))
}

/// Check /AcroForm /Fields for signature dictionaries
fn check_acroform_signatures(doc: &Document) -> Option<SignatureDetails> {
    let root = doc.catalog().ok()?;
    let acroform = root.get(b"AcroForm").ok()?.as_dict().ok()?;

    // Check /Fields array for Sig dictionaries
    if let Some(fields_obj) = acroform.get(b"Fields").ok() {
        if let Ok(fields_array) = fields_obj.as_array() {
            let mut sig_count = 0;
            let mut signer = None;

            for field_ref in fields_array {
                let obj_id = match field_ref {
                    lopdf::Object::Reference(id) => *id,
                    _ => continue,
                };
                let Ok(field_obj) = doc.get_object(obj_id) else { continue };
                let Ok(field_dict) = field_obj.as_dict() else { continue };

                if let Ok(field_type) = field_dict.get(b"FT") {
                    if field_type.as_name().map(|n| n == b"Sig").unwrap_or(false) {
                        sig_count += 1;
                        if signer.is_none() {
                            signer = extract_signer(field_dict);
                        }
                    }
                }
            }

            if sig_count > 0 {
                return Some(SignatureDetails {
                    r#type: "AcroForm/Sig".to_string(),
                    signer,
                    certificates_count: sig_count,
                });
            }
        }
    }

    None
}

/// Check /AcroForm /DOCMDP for certified signatures
fn check_docmdp(doc: &Document) -> Option<SignatureDetails> {
    let root = doc.catalog().ok()?;
    let acroform = root.get(b"AcroForm").ok()?.as_dict().ok()?;

    if acroform.get(b"DOCMDP").is_ok() {
        return Some(SignatureDetails {
            r#type: "DocMDP".to_string(),
            signer: None,
            certificates_count: 1,
        });
    }

    None
}

/// Check /Root /Permissions dictionary
fn check_permissions_dict(doc: &Document) -> Option<SignatureDetails> {
    let root = doc.catalog().ok()?;

    if root.get(b"Permissions").is_ok() {
        return Some(SignatureDetails {
            r#type: "Permissions".to_string(),
            signer: None,
            certificates_count: 1,
        });
    }

    None
}

/// Try to extract the signer name from a signature dictionary
fn extract_signer(sig_dict: &lopdf::Dictionary) -> Option<String> {
    if let Ok(name_obj) = sig_dict.get(b"Name") {
        if let Ok(name) = name_obj.as_string() {
            return Some(name.into_owned());
        }
    }
    None
}

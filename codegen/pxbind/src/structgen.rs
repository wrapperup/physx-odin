use super::{Indent, SG, UOF};
use crate::type_db::QualTypeValue;
use crate::type_db::FieldBindingValue;
use crate::writes;
use crate::writesln;

impl crate::type_db::RecBindingDef {
    pub(crate) fn emit_structgen(&self, writer: &mut String, level: u32) {
        if self.calc_layout {
            self.emit_structgen_calc(writer, level);
        } else {
            self.emit_structgen_passthrough(writer, level);
        }
    }

    fn emit_structgen_calc(&self, w: &mut String, level: u32) {
        let indent = Indent(level);
        let indent1 = Indent(level + 1);
        let indent2 = Indent(level + 2);

        let name = &self.name;

        writesln!(
            w,
            "{indent}struct physx_{name}_Pod: public physx::{name} {{"
        );
        writesln!(w, "{indent1}static void dump_layout(PodStructGen& {SG}) {{");
        writesln!(
            w,
            r#"{indent2}{SG}.begin_struct("physx_{name}_Pod", "{name}");"#
        );

		let mut emit_field = |field: &FieldBindingValue, is_base: bool| {
			let fname = &field.name;
            let cpp_type = field.kind.cpp_type();

            let fmted = format!("{cpp_type}");
            if fmted.contains("unnamed") {
                return;
            }

            if !field.is_public || field.is_reference {
                return;
            }

            writes!(w, "{indent2}{SG}.add_field(");

            // We need to handle arrays specially since they break the pattern of literally every other
            // C type since the element and array lengths are split by the identifier. Sigh.
            if let QualTypeValue::Array { element, len } = &field.kind {
                // There are a couple of cases of 2-dimensional arrays :p
                if let QualTypeValue::Array {
                    element: inner,
                    len: len1,
                } = &**element
                {
                    writes!(w, "\"{}\", \"{fname}[{len1}]", inner.c_type());
                } else {
                    writes!(w, "\"{}\", \"{fname}", element.c_type());
                }

                writes!(w, "[{len}]\"");
            } else {
                let c_type = field.kind.c_type();
                writes!(w, "\"{c_type}\", \"{fname}\"");
            }

            writesln!(
                w,
                r#", sizeof({cpp_type}), {UOF}(physx_{name}_Pod, {fname}), {is_base:?});"#,
            );
		};

        for field in &self.base_fields {
			emit_field(field, true);
        }

        for field in &self.fields {
			emit_field(field, false);
        }

        writesln!(w, "{indent2}{SG}.end_struct(sizeof(physx::{name}));");
        writesln!(w, "{indent1}}}\n{indent}}};");
        writesln!(w, "{indent}physx_{name}_Pod::dump_layout({SG});");
    }

    fn emit_structgen_passthrough(&self, w: &mut String, level: u32) {
        let indent = Indent(level);
        let cindent = Indent(1);

        writes!(
            w,
            "{indent}{SG}.pass_thru(\"{} physx_{}_Pod",
            if !matches!(self.ast_tag_used, Some(crate::consumer::Tag::Union)) {
                "struct"
            } else {
                "union"
            },
            self.name
        );

        if self.def_data.is_none() {
            writesln!(w, ";\\n\");");
            return;
        }

        writes!(w, " {{\\n");

        if self.has_vtable {
            writes!(w, "{cindent}void* vtable_;\\n");
        }

        for field in &self.fields {
            if let QualTypeValue::Array { element, len } = &field.kind {
                writes!(w, "{cindent}{} {}[{len}];\\n", element.c_type(), field.name);
            } else {
                writes!(w, "{cindent}{} {};\\n", field.kind.c_type(), field.name);
            }
        }

        writes!(w, "}};\\n\");");
    }

    // pub fn emit_rust(&self, w: &mut String, level: u32) -> bool {
    //     if self.calc_layout {
    //         return false;
    //     }

    //     let indent = Indent(level);
    //     let indent1 = Indent(level + 1);

    //     let is_union = matches!(self.ast.tag_used, Some(crate::type_db::Tag::Union));

    //     writesln!(w, "{indent}#[derive(Clone, Copy)]");

    //     if !is_union {
    //         writesln!(
    //             w,
    //             "{indent}#[cfg_attr(feature = \"debug-structs\", derive(Debug))]"
    //         );
    //     }
    //     writesln!(w, "{indent}#[repr(C)]");
    //     writesln!(
    //         w,
    //         "{indent}pub {} {} {{",
    //         if is_union { "union" } else { "struct" },
    //         self.name
    //     );

    //     if self.has_vtable && self.bases.is_empty() {
    //         writesln!(w, "{indent1}vtable_: *const std::ffi::c_void,");
    //     }

    //     for field in &self.fields {
    //         if field.is_public {
    //             writes!(w, "{indent1}pub ");
    //         }

    //         writesln!(w, "{}: {},", field.name, field.kind.rust_type());
    //     }

    //     writesln!(w, "{indent}}}");

    //     // Unions can't have derived Debug impls so we make our own
    //     if is_union {
    //         let indent2 = Indent(level + 2);
    //         writesln!(w, "{indent}#[cfg(feature = \"debug-structs\")]");
    //         writesln!(w, "{indent}impl std::fmt::Debug for {} {{", self.name);
    //         writesln!(
    //             w,
    //             "{indent1}fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{"
    //         );
    //         writesln!(w, "{indent2}f.write_str(\"{}\")", self.name);
    //         writesln!(w, "{indent1}}}");
    //         writesln!(w, "{indent}}}");
    //     }

    //     true
    // }
}

impl crate::type_db::RecBindingForward {
    pub fn emit_structgen(&self, w: &mut String, level: u32) {
        let indent = Indent(level);

        writes!(
            w,
            "{indent}{SG}.pass_thru(\"struct physx_{}_Pod;\\n\");",
            self.name
        );
    }

    // pub fn emit_rust(&self, w: &mut String, level: u32) {
    //     let indent = Indent(level);
    //     let indent1 = Indent(level + 1);

    //     writesln!(w, "{indent}#[derive(Copy, Clone)]");
    //     writesln!(w, "{indent}#[repr(C)]");
    //     writesln!(w, "{indent}pub struct {} {{", self.name);
    //     writesln!(w, "{indent1}_unused: [u8; 0],");
    //     writesln!(w, "}}");
    // }
}

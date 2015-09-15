//! This lint checks for `String` typed values (either in variables or fields)
//! that can benefit from using `std::borrow::Cow` instead.

use syntax::ast::CRATE_NODE_ID;
use rustc::lint::*;
use rustc_front::visit::Visitor;
use rustc_front::hir::*;

declare_lint!{ pub GOT_MILK, Warn,
               "look for `String`s that are only instantiated from \
               `&'static str`s or non-stringy sources, suggest Cow" }

/// a growable sequence of &Expr references
pub type ExprRefs = Vec<&Expr>;
/// either a tuple of vectors with Initializers, Assignments, Uses or None if
/// the entry is not eligible for Cow-optimization
pub type CowEntry = Option<(ExprRefs, ExprRefs, ExprRefs)>;
/// a map from node ids (that will point to either local definitions or struct
/// fields/tuple entries
pub type CowMap = NodeMap<CowEntry>;

#[derive(Copy,Clone)]
pub struct CowPass;

pub struct CowVisitor {
    map: CowMap,
    infer_ctxt: InferCtxt,
}

/// We use the visitor mainly to enter None entries for public fields, and to
/// preset fields that are non-public (perhaps we find one that is only set to
/// String values
impl Visitor<'e> for CowVisitor<'e> {
    fn visit_fn(&mut self, _: FnKind<'v>, fd: &'v FnDecl, b: &'v Block,
            _: Span, _: NodeId) {
        ExprUseVisitor::new(self, self.infer_ctxt).walk_fn(fd, b);
    }

    fn visit_item(&mut self, i: &Item) {
        let is_public = i.vis == Public;
        match i.node {
            ItemEnum(ref def, ref _generics) => {
                //TODO: How do generics fit into this?
                for variant in &def.variants {
                    match variant.node.kind {
                        TupleVariantKind(ref args) => {
                            if is_public && variant.node.vis == Public {
                                for arg in args {
                                    if is_string_type(arg.node.ty) {
                                        self.map[arg.id] = None;
                                    }
                                }
                            } else {
                                for arg in args {
                                    if is_string_type(arg.node.ty) {
                                        let _ = self.map.entry(arg.id).
                                            or_insert(Some(
                                                (vec![], vec![], vec![])));
                                    }
                                }
                            }
                        }
                        StructVariantKind(ref def) => {
                            check_struct(&mut self, def, is_public);
                        },
                    }
                }
            },
            ItemStruct(ref def, ref _generics) => {
                //TODO: How do generics fit into this?
                check_struct(&mut self, def, is_public);
            },
            _ => walk_item(self, i),
        }
    }
}

fn is_string_type(ty: &Ty) {
    if let TyPath(_, ref path) = field.node.ty.node {
        match_path(path, &STRING_PATH)
    } else { false }
}

fn check_struct(cv: &mut CowVisitor, def: &StructDef, is_public: bool) {
    for field in &def.fields {
        if !is_string_type(field.node.ty) { continue; }
        if is_public && field.node.vis == Public {
            cv.map[field.node.id] = None;
        } else {
            let _ = cv.map.entry(field.node.id).or_insert(
                Some((vec![], vec![], vec![])));
        }
    }
}

//TODO: What do we need to look at?
impl Delegate for CowPass {
//    fn consume(&mut self, consume_id: NodeId, consume_span: Span, cmt: cmt<'tcx>, mode: ConsumeMode);
//    fn matched_pat(&mut self, matched_pat: &Pat, cmt: cmt<'tcx>, mode: MatchMode);
//    fn consume_pat(&mut self, consume_pat: &Pat, cmt: cmt<'tcx>, mode: ConsumeMode);
//    fn borrow(&mut self, borrow_id: NodeId, borrow_span: Span, cmt: cmt<'tcx>, loan_region: Region, bk: BorrowKind, loan_cause: LoanCause);
//    fn decl_without_init(&mut self, id: NodeId, span: Span);
//    fn mutate(&mut self, assignment_id: NodeId, assignment_span: Span, assignee_cmt: cmt<'tcx>, mode: MutateMode);
}

impl LintPass for CowPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(GOT_MILK)
    }

    fn check_crate(&mut self, _: &Context, krate: &Crate) {
        let cv = CowVisitor{ map: NodeMap(), infer_ctxt: TODO };
        cv.visit_mod(krate.mod, krate.span, CRATE_NODE_ID);
        for (node_id, entry) in &cv.map {
            if let Some((ref inits, ref assigns, ref borrows)) = entry {
                //TODO
            }
        }
    }
}

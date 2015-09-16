//! This lint checks for `String` typed values (either in variables or fields)
//! that can benefit from using `std::borrow::Cow` instead.

use syntax::ast::{CRATE_NODE_ID, NodeId};
use syntax::codemap::Span;
use rustc::lint::*;
use rustc_front::visit::{FnKind, Visitor, walk_crate, walk_item};
use rustc_front::hir::*;
use rustc::middle::mem_categorization::{cmt, categorization};
use rustc::middle::expr_use_visitor as euv;
use rustc::middle::{infer, ty};
use rustc::util::nodemap::NodeMap;

use utils::{match_path, STRING_PATH};

declare_lint!{ pub GOT_MILK, Warn,
               "look for `String`s that are only instantiated from \
               `&'static str`s or non-stringy sources, suggest Cow" }

/// a growable sequence of &Expr references
pub type ExprRefs = Vec<NodeId>;
/// either a tuple of vectors with Initializers, Assignments, Uses or None if
/// the entry is not eligible for Cow-optimization
pub type CowEntry = Option<(ExprRefs, ExprRefs, ExprRefs)>;
/// a map from node ids (that will point to either local definitions or struct
/// fields/tuple entries
pub type CowMap = NodeMap<CowEntry>;

#[derive(Copy,Clone)]
pub struct CowPass;

pub struct CowVisitor<'v, 't: 'v> {
    map: CowMap,
    cx: &'v Context<'v, 't>,
}

impl<'v, 't: 'v> CowVisitor<'v, 't> {
    fn new(cx: &'v Context<'v, 't>) -> CowVisitor<'v, 't> {
        CowVisitor{ map: NodeMap(), cx: cx }
    }

    fn walk_crate(&mut self, krate: &'v Crate) {
        walk_crate(self, &krate);
    }
}

/// We use the visitor mainly to enter None entries for public fields, and to
/// preset fields that are non-public (perhaps we find one that is only set to
/// String values
impl<'v, 't> Visitor<'v> for CowVisitor<'v, 't> {
    fn visit_fn(&mut self, _: FnKind, fd: &FnDecl, b: &Block,
            _: Span, id: NodeId) {
        let tcx = &self.cx.tcx;
        let param_env = Some(ty::ParameterEnvironment::for_item(tcx, id));
        let infcx = infer::new_infer_ctxt(tcx, &tcx.tables, param_env, false);
        euv::ExprUseVisitor::new(self, &infcx).walk_fn(fd, b);
    }

    fn visit_item(&mut self, i: &'v Item) {
        let is_public = i.vis == Public;
        match i.node {
            ItemEnum(ref def, ref _generics) => {
                //TODO: How do generics fit into this?
                for variant in &def.variants {
                    match variant.node.kind {
                        TupleVariantKind(ref args) => {
                            if is_public && variant.node.vis == Public {
                                for arg in args {
                                    if is_string_type(&arg.ty) {
                                        let mut map = &mut self.map;
                                        map.insert(arg.id, None);
                                    }
                                }
                            } else {
                                for arg in args {
                                    if is_string_type(&arg.ty) {
                                        let _ = self.map.entry(arg.id).
                                            or_insert(Some(
                                                (vec![], vec![], vec![])));
                                    }
                                }
                            }
                        }
                        StructVariantKind(ref def) => {
                            check_struct(self, def, is_public);
                        },
                    }
                }
            },
            ItemStruct(ref def, ref _generics) => {
                //TODO: How do generics fit into this?
                check_struct(self, def, is_public);
            },
            _ => walk_item(self, i),
        }
    }
}

fn is_string_type(ty: &Ty) -> bool {
    if let TyPath(_, ref path) = ty.node {
        match_path(path, &STRING_PATH)
    } else { false }
}

fn check_struct(cv: &mut CowVisitor, def: &StructDef, is_public: bool) {
    if is_public {
        for field in &def.fields {
            if is_string_type(&field.node.ty) {
                let mut map = &mut cv.map;
                map.insert(field.node.id, None);
            }
        }
    } else {
        for field in &def.fields {
            if is_string_type(&field.node.ty) {
                let _ = cv.map.entry(field.node.id).or_insert(
                    Some((vec![], vec![], vec![])));
            }
        }
    }
}

//TODO: What do we need to look at?
impl<'v, 't: 'v> euv::Delegate<'v> for CowVisitor<'v, 't> {
    fn consume(&mut self, consume_id: NodeId, consume_span: Span,
            cmt: cmt<'v>, mode: euv::ConsumeMode) {
        //TODO
    }

    fn matched_pat(&mut self, matched_pat: &Pat, cmt: cmt<'v>,
            mode: euv::MatchMode) {
        //TODO
    }

    fn consume_pat(&mut self, consume_pat: &Pat, cmt: cmt<'v>,
            mode: euv::ConsumeMode) {
        //TODO
    }

    fn borrow(&mut self, borrow_id: NodeId, borrow_span: Span, cmt: cmt<'v>,
            loan_region: ty::Region, bk: ty::BorrowKind, loan_cause: euv::LoanCause) {
        //TODO
    }

    fn decl_without_init(&mut self, id: NodeId, span: Span) {
        //TODO
    }
    fn mutate(&mut self, assignment_id: NodeId, assignment_span: Span,
            assignee_cmt: cmt<'v>, mode: euv::MutateMode) {
        //TODO
    }
}

impl LintPass for CowPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(GOT_MILK)
    }

    fn check_crate(&mut self, cx: &Context, krate: &Crate) {
        let mut cv = CowVisitor::new(cx);
        cv.walk_crate(krate);
        //for (node_id, entry) in &cv.map {
        //    if let Some((ref inits, ref assigns, ref borrows)) = *entry {
        //        //TODO
        //    }
        //}
    }
}

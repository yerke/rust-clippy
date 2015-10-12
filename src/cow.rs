//! This lint checks for `String` typed values (either in variables or fields)
//! that can benefit from using `std::borrow::Cow` instead.

use syntax::ast::{NodeId};
use syntax::codemap::Span;
use rustc::lint::*;
use rustc_front::visit::{FnKind, Visitor, walk_crate, walk_item};
use rustc_front::hir::*;
use rustc::front::map::Node::NodeExpr;
use rustc::middle::mem_categorization::{cmt, categorization};
use rustc::middle::expr_use_visitor as euv;
use rustc::middle::{infer, ty};
use rustc::util::nodemap::NodeMap;

use utils::{match_path, STRING_PATH};

declare_lint!{ pub GOT_MILK, Allow,
               "look for `T`s instantiated from `&'static` refefences using \
                `.to_owned()` (or similar), suggest `Cow<&static, T>` or even \
                `&static T`" }

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
    cx: &'v LateContext<'v, 't>,
}

impl<'v, 't: 'v> CowVisitor<'v, 't> {
    fn new(cx: &'v LateContext<'v, 't>) -> CowVisitor<'v, 't> {
        CowVisitor{ map: NodeMap(), cx: cx }
    }

    fn walk_crate(&mut self, krate: &'v Crate) {
        walk_crate(self, &krate);
    }
    
    fn span_lint(&self, span: Span, message: &str) {
        self.cx.span_lint(GOT_MILK, span, message);
    }
    
    fn find_expr(&self, node_id: NodeId) -> Option<&Expr> {
        if let Some(NodeExpr(ref e)) = self.cx.tcx.map.find(node_id) {
            Some(e)
        } else {
            None
        }
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
        let mut vis = euv::ExprUseVisitor::new(self as &mut euv::Delegate<'t>, &infcx);
        vis.walk_fn(fd, b);
    }

    fn visit_item(&mut self, i: &'v Item) {
        let is_public = i.vis == Public;
        match i.node {
            ItemEnum(ref def, ref _generics) => {
                //TODO: How do generics fit into this?
                for variant in &def.variants {
                    match variant.node.kind {
                        TupleVariantKind(ref args) => {
                            if is_public {
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
impl<'v, 't: 'v> euv::Delegate<'t> for CowVisitor<'v, 't> {
    fn consume(&mut self, consume_id: NodeId, consume_span: Span,
            cmt: cmt<'t>, mode: euv::ConsumeMode) {
        if let Some(ref e) = self.find_expr(consume_id) {
            if let ExprAddrOf(_, _) = e.node { return; } // we get borrow la
            self.span_lint(consume_span, &format!("consume{:?} {:?} {:?}", mode,
                e, cmt));
        }
    }

    fn matched_pat(&mut self, matched_pat: &Pat, cmt: cmt<'t>,
            mode: euv::MatchMode) {
        //TODO
        self.span_lint(matched_pat.span, &format!("matched_pat{:?} {:?} {:?}", 
            mode, matched_pat, cmt));
    }

    fn consume_pat(&mut self, consume_pat: &Pat, cmt: cmt<'t>,
            mode: euv::ConsumeMode) {
        //TODO
        self.span_lint(consume_pat.span, &format!("consume_pat{:?} {:?} {:?}", 
            mode, consume_pat, cmt));
    }

    fn borrow(&mut self, borrow_id: NodeId, borrow_span: Span, cmt: cmt<'t>,
            loan_region: ty::Region, bk: ty::BorrowKind, loan_cause: euv::LoanCause) {
        //TODO
        self.span_lint(borrow_span, &format!("borrow{:?} {:?} {:?} {:?}", 
            loan_cause, cmt, bk, loan_region));        
    }

    fn decl_without_init(&mut self, id: NodeId, span: Span) {
        //TODO
        self.span_lint(span, "decl_without_init");
    }
    
    fn mutate(&mut self, assignment_id: NodeId, assignment_span: Span,
            assignee_cmt: cmt<'t>, mode: euv::MutateMode) {
        //TODO
        self.span_lint(assignment_span, &format!("mutate{:?} {:?}", 
            mode, assignee_cmt));        
    }
}

impl LintPass for CowPass {
    fn get_lints(&self) -> LintArray {
        lint_array!(GOT_MILK)
    }
}

impl LateLintPass for CowPass {
    fn check_crate(&mut self, cx: &LateContext, krate: &Crate) {
        let mut cv = CowVisitor::new(cx);
        cv.walk_crate(krate);
        //for (node_id, entry) in &cv.map {
        //    if let Some((ref inits, ref assigns, ref borrows)) = *entry {
        //        //TODO
        //    }
        //}
    }
}

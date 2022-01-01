//! Aligning nodes in their parent container.

use super::prelude::*;
use super::ParNode;

/// `align`: Configure the alignment along the layouting axes.
pub fn align(_: &mut EvalContext, args: &mut Args) -> TypResult<Value> {
    let aligns: Spec<_> = args.find().unwrap_or_default();
    let body: PackedNode = args.expect("body")?;

    let mut styles = Styles::new();
    if let Some(align) = aligns.x {
        styles.set(ParNode::ALIGN, align);
    }

    Ok(Value::block(body.styled(styles).aligned(aligns)))
}

dynamic! {
    Align: "alignment",
}

dynamic! {
    Spec<Align>: "2d alignment",
}

castable! {
    Spec<Option<Align>>,
    Expected: "1d or 2d alignment",
    @align: Align => {
        let mut aligns = Spec::default();
        aligns.set(align.axis(), Some(*align));
        aligns
    },
    @aligns: Spec<Align> => aligns.map(Some),

}

/// A node that aligns its child.
#[derive(Debug, Hash)]
pub struct AlignNode {
    /// How to align the node horizontally and vertically.
    pub aligns: Spec<Option<Align>>,
    /// The node to be aligned.
    pub child: PackedNode,
}

impl Layout for AlignNode {
    fn layout(
        &self,
        ctx: &mut LayoutContext,
        regions: &Regions,
    ) -> Vec<Constrained<Rc<Frame>>> {
        // The child only needs to expand along an axis if there's no alignment.
        let mut pod = regions.clone();
        pod.expand &= self.aligns.map_is_none();

        // Layout the child.
        let mut frames = self.child.layout(ctx, &pod);

        for ((current, base), Constrained { item: frame, cts }) in
            regions.iter().zip(&mut frames)
        {
            // Align in the target size. The target size depends on whether we
            // should expand.
            let target = regions.expand.select(current, frame.size);
            let default = Spec::new(Align::Left, Align::Top);
            let aligns = self.aligns.unwrap_or(default);
            Rc::make_mut(frame).resize(target, aligns);

            // Set constraints.
            cts.expand = regions.expand;
            cts.base = base.filter(cts.base.map_is_some());
            cts.exact = current.filter(regions.expand | cts.exact.map_is_some());
        }

        frames
    }
}
/// Represents the geometry of a tag, which is used for calculating coordinates for a drawable
/// client.
///
/// the `w` and `h` fields represent the total width and height of the tag's screen respsectively.
/// additionaly `avail_w` and `avail_h` provide the available width and height for content,
/// factoring in the padding. The `paddings` array consists of four elements, where each index
/// represents a specific padding:
///
/// - Index 0: Top padding
/// - Index 1: Bottom padding
/// - Index 2: Left padding
/// - Index 3: Right padding
///
/// For convenience, you can utilize the `TagGeometry::padding_[side]()` method to retrieve
/// specified padding value.
///
/// Typically, it's preferable to use the available width and height when drawing a client, as they
/// account for padding, reducing the likelihood of visual bugs caused by collisions. The total
/// width and height are usually reserved for drawing fullscreen clients.
#[derive(Clone)]
pub struct TagGeometry {
    /// Total width of the tag.
    pub w: u32,

    /// Total height of the tag.
    pub h: u32,

    /// Available width for content. Is the result of the total width minus the left and right
    /// padding.
    pub avail_w: u32,

    /// Available height for content. Is the result of the total width minus the top and bottom
    /// padding.
    pub avail_h: u32,

    /// Array representing the padding for each side of the tag.
    ///
    /// - Index 0: top padding
    /// - Index 1: bottom padding
    /// - Index 2: left padding
    /// - Index 3: right padding
    ///
    /// You can also use `TagGeometry::padding_[side]()` to retrieve a specified padding.
    pub paddings: [u32; 4],
}

impl TagGeometry {
    /// Creates a new `TagGeometry` instance with the given dimensions and paddings.
    pub fn new(w: u32, h: u32, paddings: [u32; 4]) -> Self {
        Self {
            w,
            h,
            paddings,
            avail_h: h - paddings[0] - paddings[1],
            avail_w: w - paddings[2] - paddings[3],
        }
    }

    /// Returns the top padding of the tag.
    #[inline(always)]
    pub fn padding_top(&self) -> u32 {
        self.paddings[0]
    }

    /// Returns the bottom padding of the tag.
    #[inline(always)]
    pub fn padding_bottom(&self) -> u32 {
        self.paddings[1]
    }

    /// Returns the left padding of the tag.
    #[inline(always)]
    pub fn padding_left(&self) -> u32 {
        self.paddings[2]
    }

    /// Returns the right padding of the tag.
    #[inline(always)]
    pub fn padding_right(&self) -> u32 {
        self.paddings[3]
    }
}


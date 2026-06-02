//! Design tokens — single source of truth for component geometry, typography,
//! spacing, radii, and motion.
//!
//! All hardcoded `px(N)` calls in component code should be replaced with
//! reads from `cx.theme().tokens().<namespace>.<field>`. Themes override
//! these values to change the visual rhythm of the entire UI without
//! touching component logic.

use std::time::Duration;

use gpui::{FontWeight, Pixels, SharedString, px};

/// Top-level container for all design tokens.
#[derive(Clone, Debug, Default)]
pub struct DesignTokens {
    pub sizes: SizeTokens,
    pub radii: RadiiTokens,
    pub spacing: SpacingTokens,
    pub typography: TypographyTokens,
    pub motion: MotionTokens,
    pub control: ControlTokens,
}

// ============================================================================
// Sizes
// ============================================================================

/// Standard control heights, icon sizes, and avatar dimensions.
#[derive(Clone, Debug)]
pub struct SizeTokens {
    // Generic control heights
    pub control_h_xxs: Pixels, // 20
    pub control_h_xs: Pixels,  // 24
    pub control_h_sm: Pixels,  // 28
    pub control_h_md: Pixels,  // 32
    pub control_h_lg: Pixels,  // 36
    pub control_h_xl: Pixels,  // 40

    // Icon sizes
    pub icon_xxs: Pixels, // 8
    pub icon_xs: Pixels,  // 10
    pub icon_sm: Pixels,  // 12
    pub icon_md: Pixels,  // 14
    pub icon_lg: Pixels,  // 16
    pub icon_xl: Pixels,  // 20

    // Avatar sizes
    pub avatar_xs: Pixels, // 20
    pub avatar_sm: Pixels, // 28
    pub avatar_md: Pixels, // 36
    pub avatar_lg: Pixels, // 44
    pub avatar_xl: Pixels, // 56

    // Status indicator dot
    pub status_dot: Pixels, // 12
}

impl Default for SizeTokens {
    fn default() -> Self {
        Self {
            control_h_xxs: px(20.),
            control_h_xs: px(24.),
            control_h_sm: px(28.),
            control_h_md: px(32.),
            control_h_lg: px(36.),
            control_h_xl: px(40.),
            icon_xxs: px(8.),
            icon_xs: px(10.),
            icon_sm: px(12.),
            icon_md: px(14.),
            icon_lg: px(16.),
            icon_xl: px(20.),
            avatar_xs: px(20.),
            avatar_sm: px(28.),
            avatar_md: px(36.),
            avatar_lg: px(44.),
            avatar_xl: px(56.),
            status_dot: px(12.),
        }
    }
}

// ============================================================================
// Radii
// ============================================================================

#[derive(Clone, Debug)]
pub struct RadiiTokens {
    pub none: Pixels, // 0
    pub xs: Pixels,   // 2
    pub sm: Pixels,   // 4
    pub md: Pixels,   // 6
    pub lg: Pixels,   // 8
    pub xl: Pixels,   // 12
    pub pill: Pixels, // 9999
}

impl Default for RadiiTokens {
    fn default() -> Self {
        Self {
            none: px(0.),
            xs: px(2.),
            sm: px(4.),
            md: px(6.),
            lg: px(8.),
            xl: px(12.),
            pill: px(9999.),
        }
    }
}

// ============================================================================
// Spacing
// ============================================================================

#[derive(Clone, Debug)]
pub struct SpacingTokens {
    pub gap_0: Pixels, // 0
    pub gap_1: Pixels, // 4
    pub gap_2: Pixels, // 8
    pub gap_3: Pixels, // 12
    pub gap_4: Pixels, // 16
    pub gap_5: Pixels, // 20
    pub gap_6: Pixels, // 24
    pub gap_7: Pixels, // 32

    pub inset_xs: Pixels, // 4
    pub inset_sm: Pixels, // 8
    pub inset_md: Pixels, // 12
    pub inset_lg: Pixels, // 16
    pub inset_xl: Pixels, // 24
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            gap_0: px(0.),
            gap_1: px(4.),
            gap_2: px(8.),
            gap_3: px(12.),
            gap_4: px(16.),
            gap_5: px(20.),
            gap_6: px(24.),
            gap_7: px(32.),
            inset_xs: px(4.),
            inset_sm: px(8.),
            inset_md: px(12.),
            inset_lg: px(16.),
            inset_xl: px(24.),
        }
    }
}

// ============================================================================
// Typography
// ============================================================================

#[derive(Clone, Debug)]
pub struct TypographyTokens {
    pub font_size_xxs: Pixels, // 10
    pub font_size_xs: Pixels,  // 11
    pub font_size_sm: Pixels,  // 12
    pub font_size_md: Pixels,  // 14
    pub font_size_lg: Pixels,  // 16
    pub font_size_xl: Pixels,  // 20
    pub font_size_2xl: Pixels, // 24

    pub line_height_tight: f32,  // 1.2
    pub line_height_normal: f32, // 1.5
    pub line_height_loose: f32,  // 1.8

    pub weight_regular: FontWeight,
    pub weight_medium: FontWeight,
    pub weight_semibold: FontWeight,
    pub weight_bold: FontWeight,

    pub family_default: SharedString,
    pub family_mono: SharedString,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            font_size_xxs: px(10.),
            font_size_xs: px(11.),
            font_size_sm: px(12.),
            font_size_md: px(14.),
            font_size_lg: px(16.),
            font_size_xl: px(20.),
            font_size_2xl: px(24.),
            line_height_tight: 1.2,
            line_height_normal: 1.5,
            line_height_loose: 1.8,
            weight_regular: FontWeight::NORMAL,
            weight_medium: FontWeight::MEDIUM,
            weight_semibold: FontWeight::SEMIBOLD,
            weight_bold: FontWeight::BOLD,
            family_default: "system-ui".into(),
            family_mono: "ui-monospace".into(),
        }
    }
}

// ============================================================================
// Motion
// ============================================================================

/// Easing function signature: takes a 0-1 progress, returns a 0-1 eased value.
pub type EasingFn = fn(f32) -> f32;

#[derive(Clone, Debug)]
pub struct MotionTokens {
    pub duration_instant: Duration,
    pub duration_very_fast: Duration, // 100ms
    pub duration_fast: Duration,      // 150ms
    pub duration_normal: Duration,    // 200ms
    pub duration_slow: Duration,      // 300ms
    pub duration_very_slow: Duration, // 400ms

    pub duration_cursor_blink: Duration,       // 500ms
    pub duration_skeleton_pulse: Duration,    // 1100ms
    pub duration_progress_spinner: Duration,  // 850ms
    pub duration_modal_fade: Duration,        // 200ms
    pub duration_modal_slide_up: Duration,    // 250ms
    pub duration_menu_open: Duration,         // 160ms
    pub duration_menu_open_fast: Duration,    // 100ms
    pub duration_menu_open_slow: Duration,    // 250ms
    pub duration_tooltip_show: Duration,      // 150ms
    pub duration_tooltip_hide: Duration,      // 100ms
    pub duration_navigator_slider: Duration,  // 200ms
    pub duration_tab_switch: Duration,        // 150ms
    pub duration_skeleton_pulse_1: Duration,  // 1100ms
    pub duration_skeleton_pulse_2: Duration,  // 1200ms
    pub duration_progress_circle: Duration,   // 900ms
    pub duration_progress_bar: Duration,      // 1500ms

    pub easing_linear: EasingFn,
    pub easing_standard: EasingFn,   // ease_in_out
    pub easing_emphasized: EasingFn, // ease_out_cubic
    pub easing_decelerate: EasingFn, // ease_out_quint
    pub easing_accelerate: EasingFn, // ease_in

    pub pulse_min_opacity: f32, // 0.55
    pub pulse_max_opacity: f32, // 0.95

    pub slide_distance: f32, // 10
    pub bounce_distance: f32, // 8
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            duration_instant: Duration::ZERO,
            duration_very_fast: Duration::from_millis(100),
            duration_fast: Duration::from_millis(150),
            duration_normal: Duration::from_millis(200),
            duration_slow: Duration::from_millis(300),
            duration_very_slow: Duration::from_millis(400),
            duration_cursor_blink: Duration::from_millis(500),
            duration_skeleton_pulse: Duration::from_millis(1100),
            duration_skeleton_pulse_1: Duration::from_millis(1100),
            duration_skeleton_pulse_2: Duration::from_millis(1200),
            duration_progress_spinner: Duration::from_millis(850),
            duration_progress_circle: Duration::from_millis(900),
            duration_progress_bar: Duration::from_millis(1500),
            duration_modal_fade: Duration::from_millis(200),
            duration_modal_slide_up: Duration::from_millis(250),
            duration_menu_open: Duration::from_millis(160),
            duration_menu_open_fast: Duration::from_millis(100),
            duration_menu_open_slow: Duration::from_millis(250),
            duration_tooltip_show: Duration::from_millis(150),
            duration_tooltip_hide: Duration::from_millis(100),
            duration_navigator_slider: Duration::from_millis(200),
            duration_tab_switch: Duration::from_millis(150),
            easing_linear: linear,
            easing_standard: ease_in_out,
            easing_emphasized: ease_out_cubic,
            easing_decelerate: ease_out_quint,
            easing_accelerate: ease_in,
            pulse_min_opacity: 0.55,
            pulse_max_opacity: 0.95,
            slide_distance: 10.0,
            bounce_distance: 8.0,
        }
    }
}

// Standard easing functions used across the library.
pub fn linear(t: f32) -> f32 {
    t
}

pub fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_out_quint(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(5)
}

pub fn ease_in(t: f32) -> f32 {
    t * t
}

// ============================================================================
// Control tokens
// ============================================================================

#[derive(Clone, Debug, Default)]
pub struct ControlTokens {
    pub button: ButtonTokens,
    pub input: InputTokens,
    pub switch: SwitchTokens,
    pub checkbox: CheckboxTokens,
    pub radio: RadioTokens,
    pub select: SelectTokens,
    pub combo_box: ComboBoxTokens,
    pub slider: SliderTokens,
    pub toast: ToastTokens,
    pub modal: ModalTokens,
    pub popover: PopoverTokens,
    pub dropdown: DropdownTokens,
    pub badge: BadgeTokens,
    pub tag: TagTokens,
    pub skeleton: SkeletonTokens,
    pub progress: ProgressTokens,
    pub avatar: AvatarTokens,
    pub tooltip: TooltipTokens,
    pub disclosure: DisclosureTokens,
    pub keybinding_input: KeybindingInputTokens,
    pub split_button: SplitButtonTokens,
    pub search_input: SearchInputTokens,
    pub number_input: NumberInputTokens,
    pub file_path_input: FilePathInputTokens,
    pub icon_button: IconButtonTokens,
    pub toggle_button: ToggleButtonTokens,
    pub empty_state: EmptyStateTokens,
    pub list_item: ListItemTokens,
    pub tree_item: TreeItemTokens,
    pub card: CardTokens,
    pub divider: DividerTokens,
    pub form: FormTokens,
    pub notification: NotificationTokens,
    pub focus_ring: FocusRingTokens,
}

// Each control gets its own token struct. Defaults are pinned to the v0.2
// hardcoded values so the visual output stays identical when the token
// migration completes.

#[derive(Clone, Debug)]
pub struct ButtonTokens {
    pub min_height: Pixels, // 36
    pub icon_button_min_size: Pixels, // 32
    pub horizontal_padding: Pixels, // 16
    pub icon_gap: Pixels, // 8
}

impl Default for ButtonTokens {
    fn default() -> Self {
        Self {
            min_height: px(36.),
            icon_button_min_size: px(32.),
            horizontal_padding: px(16.),
            icon_gap: px(8.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct InputTokens {
    pub min_height: Pixels,    // 32
    pub horizontal_padding: Pixels, // 12
    pub vertical_padding: Pixels,   // 8
    pub cursor_thickness: Pixels,   // 1
    pub focus_ring_thickness: Pixels, // 2
    pub icon_gap: Pixels,           // 8
    pub spinner_size: Pixels,       // 14
    pub text_area_min_h: Pixels,    // 120
}

impl Default for InputTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(12.),
            vertical_padding: px(8.),
            cursor_thickness: px(1.),
            focus_ring_thickness: px(2.),
            icon_gap: px(8.),
            spinner_size: px(14.),
            text_area_min_h: px(120.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SwitchTokens {
    pub track_w: Pixels,        // 34
    pub track_h: Pixels,        // 18
    pub knob_size: Pixels,      // 14
    pub padding: Pixels,        // 2
    pub border_w: Pixels,       // 1
    pub focus_border_w: Pixels, // 2
    pub duration: Duration,     // 100ms
    pub disabled_opacity: f32,  // 0.5
}

impl Default for SwitchTokens {
    fn default() -> Self {
        Self {
            track_w: px(34.),
            track_h: px(18.),
            knob_size: px(14.),
            padding: px(2.),
            border_w: px(1.),
            focus_border_w: px(2.),
            duration: Duration::from_millis(100),
            disabled_opacity: 0.5,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CheckboxTokens {
    pub box_size: Pixels,        // 16
    pub check_size: Pixels,      // 12
    pub border_w: Pixels,        // 1
    pub focus_border_w: Pixels,  // 2
    pub border_radius: Pixels,   // 4
}

impl Default for CheckboxTokens {
    fn default() -> Self {
        Self {
            box_size: px(16.),
            check_size: px(12.),
            border_w: px(1.),
            focus_border_w: px(2.),
            border_radius: px(4.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RadioTokens {
    pub ring_size: Pixels,    // 16
    pub dot_size: Pixels,     // 8
    pub border_w: Pixels,     // 1
    pub border_radius: Pixels, // 9999 (pill)
}

impl Default for RadioTokens {
    fn default() -> Self {
        Self {
            ring_size: px(16.),
            dot_size: px(8.),
            border_w: px(1.),
            border_radius: px(9999.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectTokens {
    pub min_height: Pixels,         // 32
    pub horizontal_padding: Pixels, // 12
    pub chevron_size: Pixels,       // 14
    pub menu_min_width: Pixels,     // 120
    pub menu_max_height: Pixels,    // 260
    pub item_padding_y: Pixels,     // 6
    pub item_padding_x: Pixels,     // 12
}

impl Default for SelectTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(12.),
            chevron_size: px(14.),
            menu_min_width: px(120.),
            menu_max_height: px(260.),
            item_padding_y: px(6.),
            item_padding_x: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ComboBoxTokens {
    pub min_height: Pixels,         // 32
    pub horizontal_padding: Pixels, // 12
    pub menu_width: Pixels,         // 420
    pub menu_max_height: Pixels,    // 520
    pub search_gap: Pixels,         // 8
    pub item_padding_y: Pixels,     // 6
    pub item_padding_x: Pixels,     // 12
    pub icon_size: Pixels,          // 14
}

impl Default for ComboBoxTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(12.),
            menu_width: px(420.),
            menu_max_height: px(520.),
            search_gap: px(8.),
            item_padding_y: px(6.),
            item_padding_x: px(12.),
            icon_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SliderTokens {
    pub track_h: Pixels,    // 6
    pub thumb_size: Pixels, // 16
    pub hit_padding: Pixels, // 8
    pub focus_ring: Pixels, // 2
}

impl Default for SliderTokens {
    fn default() -> Self {
        Self {
            track_h: px(6.),
            thumb_size: px(16.),
            hit_padding: px(8.),
            focus_ring: px(2.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToastTokens {
    pub min_width: Pixels,       // 220
    pub max_width: Pixels,       // 420
    pub horizontal_padding: Pixels, // 16
    pub vertical_padding: Pixels,   // 12
    pub gap: Pixels,                // 12
    pub close_icon_size: Pixels,    // 14
}

impl Default for ToastTokens {
    fn default() -> Self {
        Self {
            min_width: px(220.),
            max_width: px(420.),
            horizontal_padding: px(16.),
            vertical_padding: px(12.),
            gap: px(12.),
            close_icon_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModalTokens {
    pub min_width: Pixels,     // 320
    pub max_width: Pixels,     // 520
    pub padding: Pixels,       // 24
    pub header_gap: Pixels,    // 12
    pub footer_gap: Pixels,    // 12
    pub scrim_blur: Pixels,    // 0
    pub border_radius: Pixels, // 12
}

impl Default for ModalTokens {
    fn default() -> Self {
        Self {
            min_width: px(320.),
            max_width: px(520.),
            padding: px(24.),
            header_gap: px(12.),
            footer_gap: px(12.),
            scrim_blur: px(0.),
            border_radius: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PopoverTokens {
    pub padding_x: Pixels,  // 12
    pub padding_y: Pixels,  // 8
    pub min_width: Pixels,  // 120
    pub max_width: Pixels,  // 360
    pub max_height: Pixels, // 320
    pub arrow_size: Pixels, // 8
    pub offset: Pixels,     // 8
}

impl Default for PopoverTokens {
    fn default() -> Self {
        Self {
            padding_x: px(12.),
            padding_y: px(8.),
            min_width: px(120.),
            max_width: px(360.),
            max_height: px(320.),
            arrow_size: px(8.),
            offset: px(8.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DropdownTokens {
    pub padding_x: Pixels,  // 12
    pub padding_y: Pixels,  // 6
    pub min_width: Pixels,  // 140
    pub max_width: Pixels,  // 280
    pub max_height: Pixels, // 320
    pub item_gap: Pixels,   // 8
    pub icon_size: Pixels,  // 14
}

impl Default for DropdownTokens {
    fn default() -> Self {
        Self {
            padding_x: px(12.),
            padding_y: px(6.),
            min_width: px(140.),
            max_width: px(280.),
            max_height: px(320.),
            item_gap: px(8.),
            icon_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct BadgeTokens {
    pub min_height: Pixels,         // 20
    pub horizontal_padding: Pixels, // 8
    pub gap: Pixels,                // 4
    pub icon_size: Pixels,          // 10
}

impl Default for BadgeTokens {
    fn default() -> Self {
        Self {
            min_height: px(20.),
            horizontal_padding: px(8.),
            gap: px(4.),
            icon_size: px(10.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TagTokens {
    pub min_height: Pixels,         // 26
    pub horizontal_padding: Pixels, // 8
    pub gap: Pixels,                // 4
    pub close_button_size: Pixels,  // 16
    pub close_icon_size: Pixels,    // 10
}

impl Default for TagTokens {
    fn default() -> Self {
        Self {
            min_height: px(26.),
            horizontal_padding: px(8.),
            gap: px(4.),
            close_button_size: px(16.),
            close_icon_size: px(10.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SkeletonTokens {
    pub line_h: Pixels,    // 12
    pub line_min_w: Pixels, // 80
    pub block_min_h: Pixels, // 64
    pub border_radius: Pixels, // 6
}

impl Default for SkeletonTokens {
    fn default() -> Self {
        Self {
            line_h: px(12.),
            line_min_w: px(80.),
            block_min_h: px(64.),
            border_radius: px(6.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProgressTokens {
    pub bar_h_sm: Pixels,     // 1.5
    pub bar_h_md: Pixels,     // 2
    pub bar_h_lg: Pixels,     // 2.5
    pub bar_default_h: Pixels, // 10
    pub spinner_size_sm: Pixels, // 12
    pub spinner_size_md: Pixels, // 16
    pub spinner_size_lg: Pixels, // 20
    pub circle_size_sm: Pixels,  // 16
    pub circle_size_md: Pixels,  // 24
    pub circle_size_lg: Pixels,  // 32
    pub track_radius: Pixels,    // 9999 (pill)
    pub steps_gap: Pixels,       // 2
}

impl Default for ProgressTokens {
    fn default() -> Self {
        Self {
            bar_h_sm: px(1.5),
            bar_h_md: px(2.),
            bar_h_lg: px(2.5),
            bar_default_h: px(10.),
            spinner_size_sm: px(12.),
            spinner_size_md: px(16.),
            spinner_size_lg: px(20.),
            circle_size_sm: px(16.),
            circle_size_md: px(24.),
            circle_size_lg: px(32.),
            track_radius: px(9999.),
            steps_gap: px(2.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AvatarTokens {
    pub border_w: Pixels,         // 2
    pub status_inset: Pixels,     // 2
    pub status_dot_size: Pixels,  // 12
    pub fallback_font_size: Pixels, // 14
}

impl Default for AvatarTokens {
    fn default() -> Self {
        Self {
            border_w: px(2.),
            status_inset: px(2.),
            status_dot_size: px(12.),
            fallback_font_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TooltipTokens {
    pub padding_x: Pixels, // 8
    pub padding_y: Pixels, // 6
    pub max_width: Pixels, // 240
    pub arrow_size: Pixels, // 6
    pub offset: Pixels,    // 6
}

impl Default for TooltipTokens {
    fn default() -> Self {
        Self {
            padding_x: px(8.),
            padding_y: px(6.),
            max_width: px(240.),
            arrow_size: px(6.),
            offset: px(6.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DisclosureTokens {
    pub icon_size: Pixels, // 12
    pub chevron_size: Pixels, // 12
}

impl Default for DisclosureTokens {
    fn default() -> Self {
        Self {
            icon_size: px(12.),
            chevron_size: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeybindingInputTokens {
    pub kbd_padding_x: Pixels, // 6
    pub kbd_padding_y: Pixels, // 2
    pub kbd_min_width: Pixels, // 24
    pub separator_gap: Pixels, // 6
    pub icon_size: Pixels,     // 14
}

impl Default for KeybindingInputTokens {
    fn default() -> Self {
        Self {
            kbd_padding_x: px(6.),
            kbd_padding_y: px(2.),
            kbd_min_width: px(24.),
            separator_gap: px(6.),
            icon_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SplitButtonTokens {
    pub min_height: Pixels, // 32
    pub chevron_width: Pixels, // 24
    pub separator_w: Pixels, // 1
}

impl Default for SplitButtonTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            chevron_width: px(24.),
            separator_w: px(1.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchInputTokens {
    pub min_height: Pixels, // 32
    pub horizontal_padding: Pixels, // 12
    pub icon_size: Pixels,  // 14
    pub clear_icon_size: Pixels, // 12
    pub spinner_size: Pixels, // 14
    pub input_gap: Pixels,  // 8
}

impl Default for SearchInputTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(12.),
            icon_size: px(14.),
            clear_icon_size: px(12.),
            spinner_size: px(14.),
            input_gap: px(8.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NumberInputTokens {
    pub min_height: Pixels, // 32
    pub stepper_button_size: Pixels, // 24
    pub stepper_icon_size: Pixels,   // 10
    pub stepper_gap: Pixels,         // 2
    pub horizontal_padding: Pixels,  // 12
}

impl Default for NumberInputTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            stepper_button_size: px(24.),
            stepper_icon_size: px(10.),
            stepper_gap: px(2.),
            horizontal_padding: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FilePathInputTokens {
    pub min_height: Pixels, // 32
    pub horizontal_padding: Pixels, // 12
    pub icon_size: Pixels,  // 14
    pub action_button_size: Pixels, // 24
    pub action_gap: Pixels, // 6
}

impl Default for FilePathInputTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(12.),
            icon_size: px(14.),
            action_button_size: px(24.),
            action_gap: px(6.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct IconButtonTokens {
    pub min_size: Pixels, // 32
    pub icon_size: Pixels, // 14
}

impl Default for IconButtonTokens {
    fn default() -> Self {
        Self {
            min_size: px(32.),
            icon_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToggleButtonTokens {
    pub min_height: Pixels, // 32
    pub horizontal_padding: Pixels, // 16
    pub icon_gap: Pixels,   // 8
}

impl Default for ToggleButtonTokens {
    fn default() -> Self {
        Self {
            min_height: px(32.),
            horizontal_padding: px(16.),
            icon_gap: px(8.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EmptyStateTokens {
    pub icon_size: Pixels, // 40
    pub title_font_size: Pixels, // 16
    pub description_font_size: Pixels, // 14
    pub action_gap: Pixels, // 12
    pub vertical_padding: Pixels, // 24
}

impl Default for EmptyStateTokens {
    fn default() -> Self {
        Self {
            icon_size: px(40.),
            title_font_size: px(16.),
            description_font_size: px(14.),
            action_gap: px(12.),
            vertical_padding: px(24.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListItemTokens {
    pub min_height: Pixels,         // 36
    pub horizontal_padding: Pixels, // 12
    pub gap: Pixels,                // 8
    pub icon_size: Pixels,          // 14
    pub chevron_size: Pixels,       // 14
}

impl Default for ListItemTokens {
    fn default() -> Self {
        Self {
            min_height: px(36.),
            horizontal_padding: px(12.),
            gap: px(8.),
            icon_size: px(14.),
            chevron_size: px(14.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeItemTokens {
    pub min_height: Pixels,         // 28
    pub horizontal_padding: Pixels, // 8
    pub indent: Pixels,             // 16
    pub chevron_size: Pixels,       // 14
    pub icon_size: Pixels,          // 14
    pub drag_handle_w: Pixels,      // 4
    pub drop_indicator_h: Pixels,   // 2
}

impl Default for TreeItemTokens {
    fn default() -> Self {
        Self {
            min_height: px(28.),
            horizontal_padding: px(8.),
            indent: px(16.),
            chevron_size: px(14.),
            icon_size: px(14.),
            drag_handle_w: px(4.),
            drop_indicator_h: px(2.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CardTokens {
    pub padding: Pixels, // 16
    pub gap: Pixels,     // 12
    pub border_radius: Pixels, // 12
}

impl Default for CardTokens {
    fn default() -> Self {
        Self {
            padding: px(16.),
            gap: px(12.),
            border_radius: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DividerTokens {
    pub thickness: Pixels, // 1
    pub margin_x: Pixels,   // 0
    pub margin_y: Pixels,   // 4
}

impl Default for DividerTokens {
    fn default() -> Self {
        Self {
            thickness: px(1.),
            margin_x: px(0.),
            margin_y: px(4.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FormTokens {
    pub field_gap: Pixels,           // 12
    pub label_gap: Pixels,           // 4
    pub helper_gap: Pixels,          // 4
    pub error_gap: Pixels,           // 4
    pub group_gap: Pixels,           // 24
    pub horizontal_field_gap: Pixels, // 16
    pub horizontal_label_width: Pixels, // 120
}

impl Default for FormTokens {
    fn default() -> Self {
        Self {
            field_gap: px(12.),
            label_gap: px(4.),
            helper_gap: px(4.),
            error_gap: px(4.),
            group_gap: px(24.),
            horizontal_field_gap: px(16.),
            horizontal_label_width: px(120.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NotificationTokens {
    pub min_width: Pixels,       // 280
    pub max_width: Pixels,       // 420
    pub horizontal_padding: Pixels, // 16
    pub vertical_padding: Pixels,   // 12
    pub gap: Pixels,                // 12
    pub icon_size: Pixels,          // 16
    pub close_icon_size: Pixels,    // 14
    pub host_padding: Pixels,       // 16
    pub host_gap: Pixels,           // 12
}

impl Default for NotificationTokens {
    fn default() -> Self {
        Self {
            min_width: px(280.),
            max_width: px(420.),
            horizontal_padding: px(16.),
            vertical_padding: px(12.),
            gap: px(12.),
            icon_size: px(16.),
            close_icon_size: px(14.),
            host_padding: px(16.),
            host_gap: px(12.),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FocusRingTokens {
    pub thickness: Pixels, // 2
    pub offset: Pixels,    // 2
}

impl Default for FocusRingTokens {
    fn default() -> Self {
        Self {
            thickness: px(2.),
            offset: px(2.),
        }
    }
}
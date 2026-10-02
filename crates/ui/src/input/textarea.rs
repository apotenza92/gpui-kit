use std::rc::Rc;

use gpui::{
    App, DefiniteLength, Edges, Entity, IntoElement, Pixels, RenderOnce, SharedString,
    StyleRefinement, Styled, Window, prelude::FluentBuilder as _,
};

use super::{Input, TextareaRotation, TextareaState};
use crate::native_menu::NativeMenu;
use crate::{RoleOverride, StyledExt as _};

/// A styled ordinary multi-line text field.
#[derive(IntoElement)]
pub struct Textarea {
    state: Entity<TextareaState>,
    style: StyleRefinement,
    height: Option<DefiniteLength>,
    appearance: bool,
    bordered: bool,
    disabled: bool,
    readonly: bool,
    tab_index: isize,
    role: RoleOverride,
    aria_label: Option<SharedString>,
    editor_paddings: Option<Edges<Pixels>>,
    rotation: Option<TextareaRotation>,

    /// An optional context menu builder to allow a custom context menu.
    ///
    /// If set, this overrides the built-in context menu.
    context_menu_builder: Option<Rc<dyn Fn(NativeMenu, &mut Window, &mut App) -> NativeMenu>>,
}

impl Textarea {
    pub fn new(state: &Entity<TextareaState>) -> Self {
        Self {
            state: state.clone(),
            style: StyleRefinement::default(),
            height: None,
            appearance: true,
            bordered: true,
            disabled: false,
            readonly: false,
            tab_index: 0,
            role: RoleOverride::default(),
            aria_label: None,
            editor_paddings: None,
            rotation: None,
            context_menu_builder: None,
        }
    }

    pub fn h(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Override the internal editor padding used for multiline text layout.
    pub fn editor_paddings(mut self, paddings: Edges<Pixels>) -> Self {
        self.editor_paddings = Some(paddings);
        self
    }

    /// Rotate the editor content clockwise around its centre.
    ///
    /// The enclosing element must use [`TextareaRotation::frame_size`] while text layout retains
    /// the rotation's untransformed content size.
    pub fn rotation(mut self, rotation: TextareaRotation) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the textarea to read-only, default is `false`.
    ///
    /// Unlike [`Self::disabled`], a read-only textarea keeps the normal appearance
    /// and still can be focused, selected and copied, it only rejects the changes
    /// made by the user.
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.readonly = readonly;
        self
    }

    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = index;
        self
    }

    pub fn role(mut self, role: impl Into<RoleOverride>) -> Self {
        self.role = role.into();
        self
    }

    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Replace the built-in context menu shown on right-click.
    ///
    /// The closure receives an empty menu and returns the one to show, so it
    /// decides entirely what appears — the default items are not added.
    pub fn context_menu(
        mut self,
        f: impl Fn(NativeMenu, &mut Window, &mut App) -> NativeMenu + 'static,
    ) -> Self {
        self.context_menu_builder = Some(Rc::new(f));
        self
    }
}

impl Styled for Textarea {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl RenderOnce for Textarea {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        Input::from_state(self.state.clone())
            .appearance(self.appearance)
            .bordered(self.bordered)
            .disabled(self.disabled)
            .readonly(self.readonly)
            .tab_index(self.tab_index)
            .role(self.role)
            .textarea_rotation(self.rotation)
            .when_some(self.editor_paddings, |this, paddings| {
                this.editor_paddings(paddings)
            })
            .when_some(self.height, |this, height| this.h(height))
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .when_some(self.context_menu_builder, |this, build| {
                this.context_menu(move |menu, window, cx| build(menu, window, cx))
            })
            .refine_style(&self.style)
    }
}

use crate::{ActiveTheme, Disableable, StyledExt, h_flex};
use gpui::{
    AnyElement, App, Bounds, ClickEvent, Element, ElementId, GlobalElementId, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, MouseButton, ParentElement, Pixels, RenderOnce,
    Role, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    prelude::FluentBuilder as _,
};
use smallvec::SmallVec;

struct MenuItemAccessibility<E> {
    inner: E,
    disabled: bool,
    checked: bool,
}

impl<E> MenuItemAccessibility<E> {
    fn new(inner: E, disabled: bool, checked: bool) -> Self {
        Self {
            inner,
            disabled,
            checked,
        }
    }
}

impl<E: Element> IntoElement for MenuItemAccessibility<E> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl<E: Element> Element for MenuItemAccessibility<E> {
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Element::id(&self.inner)
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        Element::source_location(&self.inner)
    }

    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        Element::request_layout(&mut self.inner, id, inspector_id, window, cx)
    }

    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        Element::prepaint(
            &mut self.inner,
            id,
            inspector_id,
            bounds,
            request_layout,
            window,
            cx,
        )
    }

    fn paint(
        &mut self,
        id: Option<&GlobalElementId>,
        inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        Element::paint(
            &mut self.inner,
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
    }

    fn a11y_role(&self) -> Option<Role> {
        Element::a11y_role(&self.inner)
    }

    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        Element::write_a11y_info(&self.inner, node);
        if self.disabled {
            node.set_disabled();
        }
        if self.checked {
            node.set_toggled(gpui::accesskit::Toggled::True);
        }
    }

    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        Element::a11y_synthetic_children(&mut self.inner, prepaint, builder);
    }
}

#[derive(IntoElement)]
pub(crate) struct MenuItemElement {
    id: ElementId,
    group_name: SharedString,
    aria_label: Option<SharedString>,
    style: StyleRefinement,
    disabled: bool,
    checked: bool,
    selected: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_hover: Option<Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
    children: SmallVec<[AnyElement; 2]>,
}

impl MenuItemElement {
    /// Create a new MenuItem with the given ID and group name.
    pub(crate) fn new(id: impl Into<ElementId>, group_name: impl Into<SharedString>) -> Self {
        let id: ElementId = id.into();
        Self {
            id: id.clone(),
            group_name: group_name.into(),
            aria_label: None,
            style: StyleRefinement::default(),
            disabled: false,
            checked: false,
            selected: false,
            on_click: None,
            on_hover: None,
            children: SmallVec::new(),
        }
    }

    /// Set ListItem as the selected item style.
    pub(crate) fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the accessible label for the menu item.
    pub(crate) fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// Set the disabled state of the MenuItem.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the checked state exposed by this MenuItem.
    pub(crate) fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set a handler for when the MenuItem is clicked.
    pub(crate) fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set a handler for when the mouse enters the MenuItem.
    #[allow(unused)]
    pub fn on_hover(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hover = Some(Box::new(handler));
        self
    }
}

impl Disableable for MenuItemElement {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Styled for MenuItemElement {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl ParentElement for MenuItemElement {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for MenuItemElement {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let disabled = self.disabled;
        let checked = self.checked;
        let element = h_flex()
            .id(self.id)
            .role(Role::MenuItem)
            .when_some(self.aria_label, |this, label| this.aria_label(label))
            .aria_selected(self.selected)
            .group(&self.group_name)
            .gap_x_1()
            .py_1()
            .px_2()
            .text_base()
            .text_color(cx.theme().foreground)
            .relative()
            .items_center()
            .justify_between()
            .refine_style(&self.style)
            .when_some(self.on_hover, |this, on_hover| {
                this.on_hover(move |hovered, window, cx| (on_hover)(hovered, window, cx))
            })
            .when(!self.disabled, |this| {
                this.group_hover(self.group_name, |this| {
                    this.bg(cx.theme().tokens.accent)
                        .text_color(cx.theme().accent_foreground)
                })
                .when(self.selected, |this| {
                    this.bg(cx.theme().tokens.accent)
                        .text_color(cx.theme().accent_foreground)
                })
                .when_some(self.on_click, |this, on_click| {
                    this.on_mouse_down(MouseButton::Left, move |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .on_click(on_click)
                })
            })
            .when(self.disabled, |this| {
                this.text_color(cx.theme().muted_foreground)
            })
            .children(self.children);

        MenuItemAccessibility::new(element, disabled, checked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn aria_label_sets_accessible_name(_cx: &mut gpui::TestAppContext) {
        let item = MenuItemElement::new("open", "menu").aria_label("Open");

        assert_eq!(item.aria_label, Some("Open".into()));
    }

    #[gpui::test]
    fn accessibility_exposes_disabled_and_checked_state(_cx: &mut gpui::TestAppContext) {
        let element = MenuItemAccessibility::new(
            h_flex()
                .id("checked-disabled")
                .role(Role::MenuItem)
                .aria_label("Fit Width")
                .aria_selected(true),
            true,
            true,
        );
        let mut node = gpui::accesskit::Node::new(Role::MenuItem);

        element.write_a11y_info(&mut node);

        assert_eq!(node.label(), Some("Fit Width"));
        assert_eq!(node.is_selected(), Some(true));
        assert!(node.is_disabled());
        assert_eq!(node.toggled(), Some(gpui::accesskit::Toggled::True));
    }

    #[gpui::test]
    fn accessibility_omits_inapplicable_menu_item_state(_cx: &mut gpui::TestAppContext) {
        let element =
            MenuItemAccessibility::new(h_flex().id("ordinary").role(Role::MenuItem), false, false);
        let mut node = gpui::accesskit::Node::new(Role::MenuItem);

        element.write_a11y_info(&mut node);

        assert!(!node.is_disabled());
        assert_eq!(node.toggled(), None);
    }
}

pub use ftml_component_utils::{BoxCallback, PopoverSize, PopoverTrigger, PopoverTriggerType};
use ftml_component_utils::{
    Dialog, DialogSurface, Popover as ThawPopover, PopoverAppearance, PopoverPosition,
    PopoverProps as ThawPopoverProps,
};
use leptos::prelude::*;

#[slot]
pub struct OnClickModal {
    children: Children,
    #[prop(optional, into)]
    signal: RwSignal<bool>,
}

#[component]
pub fn Popover(
    #[prop(optional, into)] class: MaybeProp<String>,
    /// Action that displays the popover.
    #[prop(optional)]
    trigger_type: PopoverTriggerType,
    /// The element or component that triggers popover.
    popover_trigger: PopoverTrigger<AnyView>,
    /// Configures the position of the Popover.
    #[prop(optional,default=PopoverPosition::Top)]
    position: PopoverPosition,
    #[prop(optional)] on_click_modal: Option<OnClickModal>,
    #[prop(optional)] on_click_signal: Option<RwSignal<bool>>,
    children: Children,
    #[prop(optional, into)] appearance: MaybeProp<PopoverAppearance>,
    #[prop(optional, into)] size: Signal<PopoverSize>,
    #[prop(optional, into)] on_open: Option<BoxCallback>,
    #[prop(optional, into)] on_close: Option<BoxCallback>,
) -> impl IntoView {
    let trigger = popover_trigger.children.into_inner();

    let modal_signal = on_click_modal
        .as_ref()
        .map(|OnClickModal { signal, .. }| *signal)
        .or(on_click_signal);
    ThawPopover(ThawPopoverProps {
        class,
        trigger_type,
        position,
        appearance,
        size,
        on_open,
        on_close,
        popover_trigger: PopoverTrigger {
            children: leptos::children::ToChildren::to_children(move || {
                let t = trigger();
                if let Some(sig) = modal_signal {
                    leptos::either::Either::Left(t.add_any_attr(leptos::tachys::html::event::on(
                        leptos::tachys::html::event::click,
                        Box::new(move |_| sig.set(true)),
                    )))
                } else {
                    leptos::either::Either::Right(t)
                }
            }),
        },
        children: Box::new(move || {
            if let Some(OnClickModal { signal, children }) = on_click_modal {
                view! {
                    <Dialog open=signal>
                        <DialogSurface>//<DialogBody>
                            {children()}
                        /*</DialogBody>*/</DialogSurface>
                    </Dialog>
                }
                .into_any()
            } else {
                children().into_any()
            }
        }),
    })
    .into_any()
}

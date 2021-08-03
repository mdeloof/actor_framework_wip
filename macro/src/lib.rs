extern crate proc_macro;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn;
use syn::visit::Visit;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn event(_attr: TokenStream, input: TokenStream) -> TokenStream {

    // Get the event enum
    let event_ast = parse_macro_input!(input as syn::Item);
    let event_enum = if let syn::Item::Enum(original) = &event_ast {
        original
    } else { panic!() };
    let event_name = &event_enum.ident;
    let event_variants = &event_enum.variants;
    let event_sig_name = format_ident!("{}Sig", event_name);
    
    let event_sig_variants = event_variants.into_iter().map(|v| format_ident!("{}", v.ident));
    let event_sig_variants_path = event_sig_variants.clone().into_iter().map(|v| quote!(#event_sig_name::#v));
    let event_sig_variant_match_arms = event_variants.into_iter().map(|v| {
        //let event_sig_variant_name = format_ident!("{}Sig", v.ident);
        let event_sig_variant_name = &v.ident;
        let event_variant_name = &v.ident;
        match v.fields {
            syn::Fields::Named(_) => return quote!(#event_name::#event_variant_name {..} => Self::Sig::#event_sig_variant_name),
            syn::Fields::Unnamed(_) => return quote!(#event_name::#event_variant_name(_) => Self::Sig::#event_sig_variant_name),
            syn::Fields::Unit => return quote!(#event_name::#event_variant_name => Self::Sig::#event_sig_variant_name)
        }
    });
    
    let gen = quote! {

        use armature::event::*;
        
        #[derive(Clone, Debug)]
        pub enum #event_name {
            OnEntry,
            OnExit,
            Nop,
            Terminate,
            Detach(usize),
            Attach(Box<dyn armature::handler::Handler<Event = #event_name, Sig = #event_sig_name>>),
            #event_variants
        }

        impl armature::event::IsEvent for #event_name {
            type Event = #event_name;
            type Sig = #event_sig_name;
            

            fn new_entry_event() -> Self {
                Self::OnEntry
            }
        
            fn new_exit_event() -> Self {
                Self::OnExit
            }
        
            fn new_nop_event() -> Self {
                Self::Nop
            }

            fn new_terminate_event() -> Self {
                Self::Terminate
            }
            
            fn try_into_attach_event(self) -> Result<Box<dyn armature::handler::Handler<Event = Self::Event, Sig = Self::Sig>>, &'static str> {
                
                if let Self::Attach(handler) = self {
                    return Ok(handler)
                } else {
                    return Err("Is not an attach event");
                }
            }

            fn try_into_detach_event(self) -> Result<usize, &'static str> {
                if let Self::Detach(id) = self {
                    return Ok(id);
                } else {
                    return Err("Is not a detach event");
                }
            }

            fn as_sig(&self) -> Self::Sig {
                match self {
                    #event_name::OnEntry => Self::Sig::OnEntry,
                    #event_name::OnExit => Self::Sig::OnExit,
                    #event_name::Nop => Self::Sig::Nop,
                    #event_name::Terminate => Self::Sig::Terminate,
                    #event_name::Detach(_) => Self::Sig::Detach,
                    #event_name::Attach(_) => Self::Sig::Attach,
                    #(#event_sig_variant_match_arms),*
                }
            }
        }

        #[derive(Debug, Hash, PartialEq, Eq, Clone)]
        pub enum #event_sig_name {
            OnEntry,
            OnExit,
            Nop,
            Terminate,
            Detach,
            Attach,
            #(#event_sig_variants),*
        }
        
        impl armature::event::IsSig for #event_sig_name {
            type Sig = #event_sig_name;

            fn list_all() -> std::vec::Vec<Self::Sig> {
                return std::vec::Vec::<Self::Sig>::from([#(#event_sig_variants_path),*]);
            }

            fn is_terminate_sig(&self) -> bool {
                if let Self::Terminate = *self {
                    return true;
                } else {
                    return false;
                }
            }
            
            fn is_attach_sig(&self) -> bool {
                if let Self::Attach = *self {
                    return true;
                } else {
                    return false;
                }
            }

            fn is_detach_sig(&self) -> bool {
                if let Self::Detach = *self {
                    return true;
                } else {
                    return false;
                }
            }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn sender(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn stator_struct(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[proc_macro_attribute]
pub fn stator_states(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}

/// Visit the items inside the stator mod and extract the `stator_struct`,
/// `stator_states` and `stator_lifecycle` items.
struct StatorItemsVisitor<'ast> {
    stator_struct: Option<&'ast syn::ItemStruct>,
    stator_states: Option<&'ast syn::ItemImpl>,
    stator_lifecycle: Option<&'ast syn::ItemImpl>,
    stator_remaining_items: Vec<&'ast syn::Item>
}

impl<'ast> Visit<'ast> for StatorItemsVisitor<'ast> {

    fn visit_item(&mut self, i: &'ast syn::Item) {
        match i {
            // Find the struct item with the `stator_struct` attribute.
            syn::Item::Struct(item_struct) => {
                if item_struct.attrs.iter().any(|attr| { attr.path.is_ident("stator_struct") }) {
                    self.stator_struct = Some(item_struct);
                } else {
                    self.stator_remaining_items.push(i);
                }
            }
            // Find the impl item with the `stator_states` attribute.
            syn::Item::Impl(item_impl) => {
                if item_impl.attrs.iter().any(|attr| { attr.path.is_ident("stator_states") }) {
                    self.stator_states = Some(item_impl);
                } else if item_impl.attrs.iter().any(|attr| { attr.path.is_ident("stator_lifecycle") }) {
                    self.stator_lifecycle = Some(item_impl);
                } else {
                    self.stator_remaining_items.push(i);
                }
            }
            _ => self.stator_remaining_items.push(i)
        }
    }

}

/// Visit the fields inside the stator struct and extract the fields that
/// have an `sender` attribute.
struct StatorStructVisitor<'ast> {
    named_fields: Vec<&'ast syn::Field>,
    sender_fields: Vec<&'ast syn::Field>
}

impl<'ast> Visit<'ast> for StatorStructVisitor<'ast> {

    fn visit_fields_named(&mut self, i: &'ast syn::FieldsNamed) {
        for field in &i.named {
            self.named_fields.push(field);
            // Find struct fields with `sender` attributes.
            if field.attrs.iter().any(|attr| {attr.path.is_ident("sender")}) {
                self.sender_fields.push(field);
            }
        }
    }
}

/// Visit the states inside the stator impl and extract the init state.
struct StatorStatesVisitor<'ast> {
    init_state: Option<&'ast syn::ImplItemConst>,
    states: Vec<&'ast syn::ItemFn>
}

impl<'ast> Visit<'ast> for StatorStatesVisitor<'ast> {

    fn visit_impl_item_const(&mut self, i: &'ast syn::ImplItemConst) {
        if i.ident.to_string() == "INIT" {
            self.init_state = Some(i);
        }
    }

    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        // Find state functions that match the signatures
        assert_eq!(i.sig.inputs.len(), 2, "state handler function must have 2 arguments");
        let arg1 = &i.sig.inputs[0];
        let arg2 = &i.sig.inputs[1];
        assert!(matches!(arg1, syn::FnArg::Receiver(_)), "first arg of state handler function must be self receiver");
        assert!(matches!(arg2, syn::FnArg::Typed(_)), "second arg of state handler function must be typed");
        self.states.push(i);
    }

}

/// Visit the (optional) lifecycle methods of the stator.
struct StatorLifecycleVisitor<'ast> {
    on_attach: Option<&'ast syn::ImplItemMethod>,
    on_detach: Option<&'ast syn::ImplItemMethod>
}

impl<'ast> Visit<'ast> for StatorLifecycleVisitor<'ast> {

    fn visit_impl_item_method(&mut self, i: &'ast syn::ImplItemMethod) {
        if i.sig.ident == "on_attach" {
            self.on_attach = Some(i)
        }
        if i.sig.ident == "on_detach" {
            self.on_detach = Some(i)
        }
    }
}

/// Visit the event type as defined in the init state of the stator.
struct EventIdentVisitor<'ast> {
    event: Option<&'ast syn::PathSegment>,
    in_impl_item_const: bool,
    in_angle_bracketed_generic_arguments: bool
}

impl<'ast> Visit<'ast> for EventIdentVisitor<'ast> {

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        self.in_impl_item_const = true;
        syn::visit::visit_impl_item_const(self, node);
        self.in_impl_item_const = false;
    }

    fn visit_angle_bracketed_generic_arguments(&mut self, node: &'ast syn::AngleBracketedGenericArguments) {
        if self.in_impl_item_const {
            self.in_angle_bracketed_generic_arguments = true;
            syn::visit::visit_generic_argument(self, &node.args[1]);
            self.in_angle_bracketed_generic_arguments = false;
        } else {
            syn::visit::visit_angle_bracketed_generic_arguments(self, node);
        }  
    }

    fn visit_path_segment(&mut self, node: &'ast syn::PathSegment) {
        if self.in_angle_bracketed_generic_arguments {
            self.event = Some(node);
        } else {
            syn::visit::visit_path_segment(self, node);
        }
    }

}

/// Visit all the match arms of the state handler functions to extract the
/// stator event subscriptions.
struct EventVariantsMatchArmVisitor<'ast> {
    events: std::collections::HashSet<&'ast syn::Ident>
}

impl<'ast> Visit<'ast> for EventVariantsMatchArmVisitor<'ast> {

    fn visit_arm(&mut self, node: &'ast syn::Arm) {
        match &node.pat {
            syn::Pat::Path(path) => {
                let event_ident = &path.path.segments[1].ident;
                match event_ident.to_string().as_str() {
                    "OnEntry" | "OnExit" => {},
                    _ => {
                        self.events.insert(event_ident);
                    }
                }   
            }
            syn::Pat::TupleStruct(tuple_struct) => {
                let event_ident = &tuple_struct.path.segments[1].ident;
                match event_ident.to_string().as_str() {
                    _ => {
                        self.events.insert(event_ident);
                    }
                }
            }
            syn::Pat::Struct(pat_struct) => {
                let event_ident = &pat_struct.path.segments[1].ident;
                match event_ident.to_string().as_str() {
                    _ => {
                        self.events.insert(event_ident);
                    }
                }
            }
            syn::Pat::Wild(_) => {}
            _ => todo!("Implement additional match types: {:#?}", &node.pat)
        }
    }
}

#[proc_macro_attribute]
pub fn stator(_attr: TokenStream, input: TokenStream) -> TokenStream {

    let stator_ast = parse_macro_input!(input as syn::ItemMod);

    let mut stator_items_visitor = StatorItemsVisitor {
        stator_struct: None,
        stator_states: None,
        stator_lifecycle: None,
        stator_remaining_items: Vec::new()
    };

    stator_items_visitor.visit_item_mod(&stator_ast);

    let mut stator_struct_visitor = StatorStructVisitor {
        named_fields: Vec::new(),
        sender_fields: Vec::new()
    };

    stator_struct_visitor.visit_item_struct(stator_items_visitor
        .stator_struct
        .expect("stator doesn't have a `stator_struct` item"));

    let mut stator_states_visitor = StatorStatesVisitor {
        init_state: None,
        states: Vec::new()
    };

    stator_states_visitor.visit_item_impl(stator_items_visitor
        .stator_states
        .expect("stator doesn't have a `stator_states` item"));

    let mut stator_lifecycle_visitor = StatorLifecycleVisitor {
        on_attach: None,
        on_detach: None
    };

    if let Some(lifecycle) = stator_items_visitor.stator_lifecycle {
        stator_lifecycle_visitor.visit_item_impl(lifecycle)
    }

    let mut event_ident_visitor = EventIdentVisitor {
        event: None,
        in_impl_item_const: false,
        in_angle_bracketed_generic_arguments: false
    };

    event_ident_visitor.visit_item_impl(stator_items_visitor
        .stator_states
        .expect("stator doesn't have a `stator_states` item"));

    let mut event_variants_match_arm_visisitor = EventVariantsMatchArmVisitor {
        events: std::collections::HashSet::new()
    };

    event_variants_match_arm_visisitor.visit_item_impl(stator_items_visitor
        .stator_states
        .expect("stator doesn't have a `stator_states` item"));
    
    let stator_mod_name = &stator_ast.ident;
    let stator_struct_name = &stator_items_visitor
        .stator_struct
        .expect("stator doesn't have a `stator_struct` item")
        .ident;
    let mut stator_struct_attrs = stator_items_visitor
        .stator_struct
        .expect("stator doesn't have a `stator_struct` item")
        .clone()
        .attrs;
    stator_struct_attrs.retain(|attr| { !attr.path.is_ident("stator_struct") });


    let mut stator_struct_fields = Vec::new();
    for field in stator_struct_visitor.named_fields {
        stator_struct_fields.push(field.clone())
    }
    for stator_struct_field in &mut stator_struct_fields {
        stator_struct_field.attrs.retain(|attr| { !attr.path.is_ident("sender")});
    }

    let mut attach_senders = Vec::new();
    let mut detach_senders = Vec::new();
    for field in stator_struct_visitor.sender_fields {
        let field_ident = &field.ident;
        attach_senders.push(quote!(self.#field_ident.set_sender_component(self.get_sender_component().clone())));
        detach_senders.push(quote!(self.#field_ident.clear_sender()));
    }

    let event_name = event_ident_visitor
        .event
        .expect("no event ident found");
    let sig_name = format_ident!("{}Sig", &event_name.ident);
    let subscribers = event_variants_match_arm_visisitor
        .events.into_iter()
        .map(|v| {
            //let event_variant_sig_name = format_ident!("{}Sig", v);
            quote!(Self::Sig::#v) 
        });
    let mut stator_states_impl = stator_items_visitor.stator_states.unwrap().clone();
    stator_states_impl.attrs.retain(|attr| { !attr.path.is_ident("stator_states") });

    let on_attach_call = if let Some(_) = stator_lifecycle_visitor.on_attach {
        quote!(<#stator_struct_name>::on_attach(self))
    } else {
        quote!()
    };

    let on_detach_call = if let Some(_) = stator_lifecycle_visitor.on_detach {
        quote!(<#stator_struct_name>::on_detach(self))
    } else {
        quote!()
    };

    let stator_lifecycle_impl = if let Some (lifecycle_impl_item) = stator_items_visitor.stator_lifecycle {
        let mut impl_item = lifecycle_impl_item.clone();
        impl_item.attrs.retain(|attr| { !attr.path.is_ident("stator_lifecycle") });
        quote!(#impl_item)
    } else {
        quote!()
    };
    
    let stator_remaining_items = stator_items_visitor.stator_remaining_items;

    let gen = quote! {

        mod #stator_mod_name {

            use armature::stator::*;
            use armature::event::*;
            use armature::handler::*;
            use armature::sender::*;

            #(#stator_struct_attrs)*
            pub struct #stator_struct_name {
                stator_component: StatorComponent<Self, #event_name>,
                handler_component: HandlerComponent<#sig_name>,
                sender_component: SenderComponent<#event_name>,
                #(#stator_struct_fields),*
            }

            #stator_states_impl

            impl armature::stator::Stator for #stator_struct_name {

                const INIT: armature::stator::State<Self, Event> = Self::INIT;

                fn get_stator_component_mut(&mut self) -> &mut StatorComponent<Self, Event> {
                    &mut self.stator_component
                }

                fn get_stator_component(&self) -> &StatorComponent<Self, Self::Event> {
                    &self.stator_component
                }
            
            }
    
            impl armature::handler::Handler for #stator_struct_name {
                type Sig = #sig_name;
        
                fn get_handler_component_mut(&mut self) -> &mut HandlerComponent<Self::Sig> {
                    &mut self.handler_component
                }
            
                fn get_handler_component(&self) -> &HandlerComponent<Self::Sig> {
                    &self.handler_component
                }
            
                fn on_attach(&mut self) {
                    #(#attach_senders);*;
                    #on_attach_call;
                }
            
                fn on_detach(&mut self) {
                    #(#detach_senders);*;
                    #on_detach_call;
                }
            
                fn init(&mut self) {
                    Stator::init(self);
                }
            
                fn handle(&mut self, event: &Self::Event) {
                    Stator::handle(self, event);
                }
            
                fn get_init_subscriptions(&self) -> Vec<Self::Sig>  {
                    Vec::from([#(#subscribers),*])
                }
    
            }
            
            impl armature::sender::Sender for #stator_struct_name {
                type Event = #event_name;

                fn get_sender_component_mut(&mut self) -> &mut SenderComponent<Self::Event> {
                    &mut self.sender_component
                }

                fn get_sender_component(&self) -> &SenderComponent<Self::Event> {
                    &self.sender_component
                }
    
            }

            #stator_lifecycle_impl

            #(#stator_remaining_items)*

        }
        
    };

    gen.into()

}
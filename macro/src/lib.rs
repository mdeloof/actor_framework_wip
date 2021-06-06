extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn;
use syn::parse_macro_input;


/// Transform struct into stator
/// 
/// A stator is a stateful actor processes events
#[proc_macro_attribute]
pub fn stator(attr: TokenStream, input: TokenStream) -> TokenStream {

    // Get the name of the event enum
    let args_ast = parse_macro_input!(attr as syn::AttributeArgs);
    let event_ast = &(args_ast[0]);
    let event_name = if let syn::NestedMeta::Meta(event) = event_ast {
        event
    } else { panic!() };

    // Get the stator struct
    let stator_ast = parse_macro_input!(input as syn::Item);
    let stator_struct = if let syn::Item::Struct(original) = &stator_ast {
        original
    } else { panic!() };
    // Get the stator name
    let stator_name = &stator_struct.ident;
    // Get the user-defined stator fields
    let stator_fields = &stator_struct.fields;
    let stator_named_fields = if let syn::Fields::Named(named_fields) = stator_fields {
        &named_fields.named
    } else { panic!() };
    
    let mut stator_args = vec![];
    let mut stator_args_name = vec![];
    for field in stator_fields {
        let arg_name = if let Some(ident) = &field.ident {
            ident
        } else { panic!() };
        let arg_type = if let syn::Type::Path(field_type) = &field.ty {
            &field_type.path
        } else { panic!() };
        stator_args.push(quote!(#arg_name: #arg_type));
        stator_args_name.push(quote!(#arg_name));
    }

    let gen = quote! {
        pub struct #stator_name {
            pub id: Option<usize>,
            pub state: State<Self, #event_name>,
            pub event_sender: Option<mpsc::UnboundedSender<Envelope<#event_name>>>,
            pub defered_event_queue: VecDeque<#event_name>,
            #stator_named_fields
        }

        impl Stator<#event_name> for #stator_name {

            fn get_id(&self) -> Option<usize> {
                self.id
            }
        
            fn get_state(&mut self) -> State<Self, #event_name> {
                self.state
            }
        
            fn set_state(&mut self, state: State<Self, #event_name>) {
                self.state = state
            }
        
            fn get_event_sender(&mut self) -> &mut Option<mpsc::UnboundedSender<Envelope<#event_name>>> {
                &mut self.event_sender
            }
        
            fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<#event_name>>) {
                self.event_sender = Some(event_sender);
            }
        
            fn get_defered_event_queue(&mut self) -> &mut VecDeque<#event_name> {
                &mut self.defered_event_queue
            }
        
        }

        impl Handler<#event_name> for #stator_name {
    
            fn init(&mut self) {
                Stator::init(self);
            }
        
            fn handle(&mut self, event: &#event_name) {
                Stator::handle(self, event);
            }
        
            fn set_event_sender(&mut self, event_sender: mpsc::UnboundedSender<Envelope<#event_name>>) {
                Stator::set_event_sender(self, event_sender);
            }
        
            fn set_id(&mut self, id: usize) {
                self.id = Some(id);
            }

        }

        impl #stator_name { 
            /// New function
            pub fn new(#(#stator_args),*) -> Self {
                Self {
                    id: None,
                    state: Led::off,
                    event_sender: None,
                    defered_event_queue: VecDeque::<MyEvent>::new(),
                    #(#stator_args_name),*
                }
            }
        }
    }; 
    gen.into()
}


#[proc_macro_attribute]
pub fn event(_attr: TokenStream, input: TokenStream) -> TokenStream {

    // Get the event enum
    let event_ast = parse_macro_input!(input as syn::Item);
    let event_enum = if let syn::Item::Enum(original) = &event_ast {
        original
    } else { panic!() };
    let event_name = &event_enum.ident;
    let event_variants = &event_enum.variants;
    
    let gen = quote! {
        #[derive(Copy, Clone)]
        pub enum #event_name {
            OnEntry,
            OnExit,
            Nop,
            #event_variants
        }

        impl Event for #event_name {

            fn get_entry_event() -> Self {
                Self::OnEntry
            }
        
            fn get_exit_event() -> Self {
                Self::OnExit
            }
        
            fn get_nop_event() -> Self {
                Self::Nop
            }
        
        }
    };
    gen.into()
}
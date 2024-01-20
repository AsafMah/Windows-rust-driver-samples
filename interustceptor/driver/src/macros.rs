macro_rules! kernel_callback {
    (fn $fn_name:ident($($param_name:ident: $param_type:ty),*) -> $ret_type:ty {
        $($body:tt)*
    }) => {
        #[link_section = "PAGE"]
        $vis unsafe extern "C" fn $fn_name($($param_name: $param_type),*) -> $ret_type {
            paged_code!();
            $($body)*
        }
    };
}

macro_rules! driver_entry {{ $($body:tt)* } => {
        #[link_section = "INIT"]
        #[export_name = "DriverEntry"] // WDF expects a symbol with the name DriverEntry
        extern "system" fn driver_entry(
            driver: &mut DRIVER_OBJECT,
            registry_path: PCUNICODE_STRING,
        ) -> NTSTATUS {
            $($body)*
        }
    };
}


macro_rules! init_object {
    ($type:ty, { $($field:ident : $value:expr),* $(,)* }) => {{
        let mut object = <$type>::default();
        $(object.$field = $value;)*
        object.Size = core::mem::size_of::<$type>() as ULONG;
        object
    }};
}


mod test {
    use wdk_sys::{_WDF_EXECUTION_LEVEL, _WDF_SYNCHRONIZATION_SCOPE, ULONG, WDF_OBJECT_ATTRIBUTES};

    #[test]
    fn test_init_object() {
        let attributes = init_object!(WDF_OBJECT_ATTRIBUTES, {
            ExecutionLevel: _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent,
            SynchronizationScope: _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent,
        });

        assert_eq!(attributes.Size, core::mem::size_of::<WDF_OBJECT_ATTRIBUTES>() as ULONG);
        assert_eq!(attributes.ExecutionLevel, _WDF_EXECUTION_LEVEL::WdfExecutionLevelInheritFromParent);
        assert_eq!(attributes.SynchronizationScope, _WDF_SYNCHRONIZATION_SCOPE::WdfSynchronizationScopeInheritFromParent);
        assert_eq!(attributes.ParentObject, core::ptr::null_mut());
    }
}

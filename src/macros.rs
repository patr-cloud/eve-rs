#[macro_export]
macro_rules! default_middleware {
    ($($fn_name:ident),+ $(,)?) => (
        default_middleware!((); $($fn_name),+)
    );
    ($state:ty; $($fn_name:ident),+ $(,)?) => (
        [
            $($crate::DefaultMiddleware::<$state>::new(|context, next| {
                Box::pin(async move {
                    $fn_name(context, next).await
                })
            })),+
        ]
    );
}

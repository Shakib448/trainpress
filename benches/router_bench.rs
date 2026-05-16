use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use trainpress::router::Router;
use trainpress::Request;
use trainpress::handler::{Handler, into_handler};
use hyper::Method;

fn dummy_handler() -> Handler {
    into_handler(|_req: Request| async { "" })
}

fn router_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("router");

    // Benchmark: Simple route matching
    group.bench_function("simple_route_match", |b| {
        let mut router = Router::new();
        router.add(Method::GET, "/", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/"))
        });
    });

    // Benchmark: Route with single parameter
    group.bench_function("single_param_match", |b| {
        let mut router = Router::new();
        router.add(Method::GET, "/users/{id}", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/users/123"))
        });
    });

    // Benchmark: Route with multiple parameters
    group.bench_function("multi_param_match", |b| {
        let mut router = Router::new();
        router.add(Method::GET, "/users/{user_id}/posts/{post_id}", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/users/123/posts/456"))
        });
    });

    // Benchmark: Deep nested route
    group.bench_function("deep_nested_route", |b| {
        let mut router = Router::new();
        router.add(Method::GET, "/api/v1/users/{id}/posts/{post_id}/comments/{comment_id}", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/api/v1/users/123/posts/456/comments/789"))
        });
    });

    // Benchmark: Router with many routes
    group.bench_function("many_routes_lookup", |b| {
        let mut router = Router::new();

        // Add 100 routes
        for i in 0..100 {
            router.add(Method::GET, &format!("/route{}", i), dummy_handler());
        }
        router.add(Method::GET, "/target", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/target"))
        });
    });

    // Benchmark: Route not found
    group.bench_function("route_not_found", |b| {
        let mut router = Router::new();
        router.add(Method::GET, "/exists", dummy_handler());

        b.iter(|| {
            black_box(router.find(&Method::GET, "/does-not-exist"))
        });
    });

    // Benchmark: Different HTTP methods
    for method in &[Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH] {
        group.bench_with_input(
            BenchmarkId::new("method_routing", format!("{}", method)),
            method,
            |b, method| {
                let mut router = Router::new();
                router.add(Method::GET, "/resource", dummy_handler());
                router.add(Method::POST, "/resource", dummy_handler());
                router.add(Method::PUT, "/resource", dummy_handler());
                router.add(Method::DELETE, "/resource", dummy_handler());
                router.add(Method::PATCH, "/resource", dummy_handler());

                b.iter(|| {
                    black_box(router.find(method, "/resource"))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, router_benchmarks);
criterion_main!(benches);

#include <bson/bson.h>

#include <uv.h>

#include <catch2/benchmark/catch_benchmark_all.hpp>
#include <catch2/catch_test_macros.hpp>
#include <mongoac/bson.h>
#include <mongoac/client-fwd.h>
#include <mongoac/client.h>
#include <mongoac/collection-fwd.h>
#include <mongoac/collection.h>
#include <mongoac/database.h>
#include <mongoac/error-fwd.h>
#include <mongoac/error.h>
#include <mongoac/future-fwd.h>
#include <mongoac/future.h>
#include <mongoac/runtime-fwd.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

#include <algorithm>
#include <array>
#include <atomic>
#include <chrono>
#include <condition_variable>
#include <cstddef>
#include <cstdint>
#include <iterator>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <thread>
#include <utility>
#include <vector>

// For convenience: not strictly needed for the libuv example, but reduces boilerplate and verbosity.
using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_bson_view;
using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;
using mongoac::test_util::owning_ptr;
using mongoac::test_util::owning_string;
using mongoac::test_util::to_mongoac;

namespace
{

// For convenience (allow Catch2 exceptions during setup).
struct uv_loop_type {
   uv_loop_t _loop = {};

   ~uv_loop_type()
   {
      CHECK(uv_loop_close(&_loop) == 0);
   }

   uv_loop_type()
   {
      REQUIRE(uv_loop_init(&_loop) == 0);
   }
};

// The mongoac worker thread's state.
struct mongoac_loop_ctx {
   struct work_type;

   // Required by C to be a function pointer; C++ can use `std::function<T>`.
   using on_ready_cb = void (*)(mongoac_loop_ctx &ctx, work_type &work) noexcept;

   // Associate every mongoac future with an `on_ready` completion callback.
   struct work_type {
      owning_ptr<mongoac_future_t> future;

      // Shared pointer for ownership simplicity. This fulfills the role of the "operation state" in `std::execution`.
      // Real-world implementations may use void* in C, void* / std::unique_ptr<void, D> / std::shared_ptr<void> in
      // C++, (non-owning) std::coroutine_handle<> with C++20 Coroutines, or whatever abstraction and ownership model
      // is appropriate for the given task and context.
      std::shared_ptr<void> data;

      // The completion handler. This fulfills the role of the "receiver" in `std::execution` (typically
      // specified via `ex::then()`). Real-world implementations may use alternative methods of completion, such as
      // function pointers, `std::function<T>`, vtables, or coroutine resumption (including separate
      // value/error/cancel handlers) as appropriate for the given task and context.
      on_ready_cb on_ready = nullptr;
   };

   // A single mutex for simplicity. Coordinates scheduling tasks on the mongoac worker thread (progress) vs. on the
   // main thread (completion). Real-world implementations may use more efficient patterns to reduce contention, such
   // as lock-free queues, or avoid synchronization by executing completion handlers on the worker thread(s) instead.
   std::mutex _mtx;
   std::condition_variable _cv;              // Wake up the mongoac worker thread.
   std::vector<work_type> _pending;          // Pending tasks processed by the mongoac worker thread.
   std::vector<work_type> _ready;            // Ready tasks to be processed by the main thread.
   std::vector<work_type> _completing;       // Ready tasks being completed by the main thread (without blocking `mtx`).
   std::atomic<bool> _is_completing = false; // Tasks are queued or being completed on the main thread.
   std::atomic<bool> _worker_failed = false; // Signal unexpected mongoac worker thread runtime failure.

   uv_loop_t *_loop = nullptr; // The main thread's event loop.
   uv_work_t _work = {};       // The mongoac worker thread's event loop.
   uv_timer_t _timer = {};     // For test timeouts.
   uv_async_t _waker = {};     // Wake up the main loop when a task is ready for completion.

   // For testing purposes: support test timeouts.
   std::atomic<bool> _stop_requested = false;

   // Dedicated ErrorT for the main thread.
   owning_ptr<mongoac_error_t> error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   // Reusable ClientT (primarily used by the main thread).
   owning_ptr<mongoac_client_t> client = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_new(to_mongoac("mongodb://localhost:27017?serverSelectionTimeoutMS=1000"), error),
      &mongoac_client_destroy);

   // Reusable RuntimeT (primarily used by the worker thread).
   owning_ptr<mongoac_runtime_t> runtime =
      make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   ~mongoac_loop_ctx()
   {
      // Block on the client's background tasks (e.g. `endSessions`) so that destroying the client does not
      // leave them running.
      mongoac_client_shutdown(client, nullptr);

      uv_timer_stop(&_timer);

      // Close all remaining handles to trigger cleanup.
      uv_close(reinterpret_cast<uv_handle_t *>(&_timer), nullptr);
      uv_close(reinterpret_cast<uv_handle_t *>(&_waker), nullptr);

      // Drain the close callbacks before the loop is closed.
      uv_run(_loop, UV_RUN_NOWAIT);
   }

   mongoac_loop_ctx(uv_loop_t *loop) : _loop(loop)
   {
      // Completion callbacks for ready futures are lazily scheduled on the main thread.
      {
         static constexpr auto on_wake_cb = +[](uv_async_t *wake) noexcept {
            auto &ctx = *static_cast<mongoac_loop_ctx *>(wake->data);

            // Swap ready tasks into the (empty) completion queue.
            {
               auto const lock = std::lock_guard(ctx._mtx);
               ctx._completing.swap(ctx._ready);
            }

            // Invoke completion callbacks on the main thread.
            for (auto &work : ctx._completing) {
               if (work.on_ready) {
                  work.on_ready(ctx, work);
               }
            }

            // Destroy completed tasks (releasing per-task state); reuse the buffer for the next swap.
            ctx._completing.clear();

            // Notify mongoac worker thread that main loop is ready to process the next batch of ready tasks.
            {
               auto const lock = std::lock_guard(ctx._mtx);
               ctx.set_completing(false);
            }
            ctx._cv.notify_one();
         };
         _waker.data = this;
         CHECK(uv_async_init(loop, &_waker, on_wake_cb) == 0);
      }

      // Enforce a loop timeout for testing purposes.
      CHECK(uv_timer_init(loop, &_timer) == 0);
      _timer.data = this;
   }

   // Add a (mongoac) task to the pending queue. Supporting arbitrary tasks are out-of-scope for this example.
   void
   queue(owning_ptr<mongoac_future_t> future, std::shared_ptr<void> data, on_ready_cb on_ready) noexcept
   {
      auto const lock = std::lock_guard(_mtx);
      _pending.push_back({std::move(future), std::move(data), on_ready});
   }

   // Run all scheduled tasks to completion (or timeout).
   void
   run(std::optional<std::chrono::milliseconds> timeout = {}) noexcept
   {
      // Reset main loop state to allow repeated calls to `this->run()`.
      this->reset(timeout);

      // Schedule (dispatch) the mongoac worker thread.
      _work.data = this;
      CHECK(uv_queue_work(
               _loop, &_work, &loop, +[](uv_work_t *work, int) noexcept {
                  // The worker thread completed; stop the main loop.
                  auto &ctx = *static_cast<mongoac_loop_ctx *>(work->data);
                  uv_stop(ctx._loop);
               }) == 0);

      // Block until all work is complete or main loop timeout.
      // The main thread sleeps until a task is ready, then executes the `on_ready` callback for the completed
      // tasks. All progress is on the (mongoac) worker thread.
      uv_run(_loop, UV_RUN_DEFAULT);

      // Validate the mongoac worker thread itself did not encounter an error.
      CHECK_FALSE(_worker_failed.load(std::memory_order_relaxed));

      // For test purposes only: a stop request is raised by the watchdog on timeout.
      CHECK_FALSE(this->stop_requested());
   }

   // The mongoac worker thread's event loop.
   static void
   loop(uv_work_t *work) noexcept
   {
      auto &ctx = *static_cast<mongoac_loop_ctx *>(work->data);

      // Dedicated ErrorT for the mongoac worker thread.
      auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

      // Local non-owning copy of pending futures compared and synced with `ctx._pending`.
      std::vector<mongoac_future_t *> pending;

      // Continue until interruption (or explicit `return;`).
      while (!ctx.stop_requested()) {
         {
            auto lock = std::unique_lock(ctx._mtx);

            // Wait for newly scheduled tasks, for no tasks to remain, or for a stop request.
            ctx._cv.wait(lock, [&] { return !ctx._pending.empty() || !ctx.tasks_remain() || ctx.stop_requested(); });

            // No tasks remaining, or a stop request (e.g. test timeout).
            if (!ctx.tasks_remain() || ctx.stop_requested()) {
               return;
            }

            // New pending tasks may have been scheduled.
            if (ctx._pending.size() != pending.size()) {
               pending.clear();
               pending.reserve(ctx._pending.size());
               for (auto const &item : ctx._pending) {
                  pending.push_back(item.future.get());
               }
            }
         }

         // This choice and method of "progress function" invocation determines the entire shape of how the mongoac
         // library integrates into the user's execution model (libuv, std::execution, etc.).
         //
         // There are many options available for a real-world implementation to drive progress on one or more
         // futures:
         //  - periodically invoking `make_progress()`,
         //  - blocking until all futures are ready, or
         //  - blocking until at least one future is ready.
         // Similarly, there are many options available to handle ready futures:
         //  - poll every future per iteration for readiness,
         //  - poll/block-on only the first future for readiness,
         //  - block on all futures at a time, or
         //  - block on only a single future at a time.
         // All of these options have trade-offs w.r.t. implementation complexity, latency, and throughput. Identifying
         // which approach is most suitable for a given real-world implementation, including adjustable parameters such
         // as runtime timeout windows, will require workload-dependent benchmarks and deliberate adjustments. The
         // mongoac library does not prescribe any particular execution model.
         //
         // For this example, the "block until all futures are ready" approach is used for its simplicity (move all
         // ready futures to main thread for completion), with an arbitrary 100ms timeout to allow new pending tasks to
         // be added to the pending queue. An alternative method may handle one future at a time, execute the completion
         // callback immediately on the worker thread (rather than on the main thread), or any number of other
         // approaches, including single-threaded execution models which must interleave progress and completion on the
         // same thread.
         mongoac_runtime_block_on_all_with_timeout(ctx.runtime, pending.data(), pending.size(), 100, error);

         switch (mongoac_error_code(error)) {
         case MONGOAC_ERROR_CODE_OK:
            // All futures are ready.
            break;
         case MONGOAC_ERROR_CODE_TIMEOUT:
            // Allow new pending tasks to be included in the next iteration.
            continue;
         default:
            // These examples do not support an error channel: abort the loop on failure.
            // Note: this assertion requires CATCH_CONFIG_THREAD_SAFE_ASSERTIONS.
            FAIL_CHECK("mongoac worker failed: " << owning_string(mongoac_error_message(error)).view());
            ctx._worker_failed.store(true, std::memory_order_relaxed);
            return;
         }

         // Every pending future is ready: transfer to the main thread for completion.
         {
            auto const lock = std::lock_guard(ctx._mtx);
            std::move(ctx._pending.begin(), ctx._pending.end(), std::back_inserter(ctx._ready));
            ctx._pending.clear();
            pending.clear();
         }

         // Notify main thread that there are ready tasks awaiting completion.
         ctx.set_completing(true);
         CHECK(uv_async_send(&ctx._waker) == 0);
      }
   }

   [[nodiscard]] bool
   stop_requested() const noexcept
   {
      return _stop_requested.load(std::memory_order_relaxed);
   }

   void
   request_stop(bool v) noexcept
   {
      _stop_requested.store(v, std::memory_order_relaxed);
   }

   [[nodiscard]] bool
   is_completing() const noexcept
   {
      return _is_completing.load(std::memory_order_relaxed);
   }

   void
   set_completing(bool v) noexcept
   {
      _is_completing.store(v, std::memory_order_relaxed);
   }

   // Precondition: `_mtx` is locked.
   [[nodiscard]] bool
   tasks_remain() const noexcept
   {
      // Either there are:
      // - pending tasks making progress,
      // - ready tasks scheduled to be completed, or
      // - tasks being completed.
      return !_pending.empty() || !_ready.empty() || this->is_completing();
   }

   void
   reset(std::optional<std::chrono::milliseconds> timeout) noexcept
   {
      CHECK(uv_timer_stop(&_timer) == 0);

      // For test purposes: apply a main loop timeout to prevent indefinite execution.
      if (timeout) {
         uv_timer_cb const timer_cb = +[](uv_timer_t *timer) noexcept {
            auto &ctx = *static_cast<mongoac_loop_ctx *>(timer->data);
            {
               auto const lock = std::lock_guard(ctx._mtx);
               ctx.request_stop(true);
            }
            ctx._cv.notify_one();
         };

         CHECK(*timeout >= std::chrono::milliseconds());
         CHECK(uv_timer_start(&_timer, timer_cb, static_cast<std::uint64_t>(timeout->count()), 0) == 0);
      }

      this->request_stop(false);
   }
};

// Return true only when `{"ok": 1.0}`.
static bool
ping_ok(mongoac_bson_view_t reply) noexcept
{
   // No reply.
   if (!reply.ptr) {
      return false;
   }

   bson_t result = {};
   if (!bson_init_static(&result, reply.ptr, reply.len)) {
      return false;
   }

   bson_iter_t iter = {};
   return bson_iter_init_find(&iter, &result, "ok") && bson_iter_as_double(&iter) == 1.0;
}

} // namespace

// libuv recommends a dedicated thread per external I/O library and integration via thread pool work scheduling.
TEST_CASE("worker thread", "[examples][libuv]")
{
   // The main loop (on this thread).
   uv_loop_type loop_owner;
   uv_loop_t &loop = loop_owner._loop;

   // The mongoac worker thread state (the thread itself is spawned by `run()`).
   auto ctx = mongoac_loop_ctx(&loop);

   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const db = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(ctx.client, to_mongoac("admin"), nullptr, error),
                                           &mongoac_database_destroy);
   auto const ping_owner = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
   auto const ping = make_bson_view(ping_owner.get());

   static constexpr auto noop_deleter = [](auto *) noexcept {};

   SECTION("do nothing")
   {
      SUCCEED("do nothing");
   }

   SECTION("no work")
   {
      ctx.run(std::chrono::seconds(3));
   }

   SECTION("single future")
   {
      struct result_type {
         std::thread::id main_thread = std::this_thread::get_id();
         int invoked = 0;
      } result;

      // In this example, the completion callback is invoked only once the given future is ready.
      static constexpr auto on_shutdown = +[](mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept {
         mongoac_future_get_void(work.future, ctx.error);
         CHECK_MONGOAC_OK(ctx.error);

         auto &result = *static_cast<result_type *>(work.data.get());

         // In this example, the completion callback is always executed synchronously on the main thread.
         CHECK(result.main_thread == std::this_thread::get_id());

         // Therefore, thread-safety is not required.
         result.invoked += 1;
      };

      // Return via non-owning reference to local `result`.
      ctx.queue(make_owning_ptr(mongoac_client_shutdown_async(ctx.client, ctx.error), &mongoac_future_destroy),
                std::shared_ptr<result_type>(&result, noop_deleter),
                on_shutdown);

      ctx.run(std::chrono::seconds(3));

      CHECK(result.invoked == 1);
   }

   // Correctness check for equivalent benchmark.
   SECTION("100 sequential pings (sync)")
   {
      int count = 0;

      for (auto i = 0; i < 100; ++i) {
         auto const reply = owning_bson(mongoac_database_run_command(db, nullptr, ping, nullptr, ctx.error));
         REQUIRE_MONGOAC_OK(ctx.error);
         count += ping_ok(reply) ? 1 : 0;
      }

      CHECK(count == 100);
   }

   // Correctness check for equivalent benchmark.
   SECTION("100 concurrent pings (async)")
   {
      int count = 0;

      // These examples use captureless lambdas (as function pointers) + `noexcept` to express C compatibility without
      // requiring namespace-scoped functions that would reduce readability due to scoping and ordering.
      static constexpr auto on_ping = +[](mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept {
         // Completion callback is only invoked when the future is ready.
         CHECK(mongoac_future_is_ready(work.future));

         auto const reply = mongoac_future_get_bson(work.future, ctx.error);
         CHECK_MONGOAC_OK(ctx.error);

         *static_cast<int *>(work.data.get()) += ping_ok(reply) ? 1 : 0;
      };

      for (auto i = 0; i < 100; ++i) {
         ctx.queue(make_owning_ptr(mongoac_database_run_command_async(db, nullptr, ping, nullptr, ctx.error),
                                   &mongoac_future_destroy),
                   std::shared_ptr<int>(&count, noop_deleter),
                   on_ping);
      }

      ctx.run(std::chrono::seconds(3));

      CHECK(count == 100);
   }

   // Perform the following sequence of operations:
   //
   // * --> (insert `{v: 0}`: _id0) --> (update _id0 to `{v: 1}`) --> * --> (find "v" for _id0) --> sum "v".
   //   \-> (insert `{v: 0}`: _id1) --> (update _id1 to `{v: 2}`) -/    \-> (find "v" for _id1) -/
   //   \-> (insert `{v: 0}`: _id2) --> (update _id2 to `{v: 3}`) -/    \-> (find "v" for _id2) -/
   //
   // Each successive operation is scheduled only after the completion of its dependent operation via completion
   // callback and uses the result(s) of the previous operation (e.g. `_id`) as input for the next operation.
   //
   // This example further demonstrates how each task may create and forward its own state on-demand. A real-world
   // implementation may use higher-level abstractions to express and manage task composition in a more expressive
   // manner, via `std::execution` and C++20 Coroutines. The mongoac library does not prescribe any particular
   // composition method.
   SECTION("insert_find_update")
   {
      static constexpr auto make_owning_bson =
         +[](bson_t *bson) noexcept { return make_owning_ptr(bson, &bson_destroy); };

      // Clean test state.
      auto const db =
         REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(
                                    ctx.client, to_mongoac("mongoac-worker_thread-insert_update_find"), nullptr, error),
                                 &mongoac_database_destroy);
      auto const coll = REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, to_mongoac("coll"), nullptr, error),
                                                &mongoac_collection_destroy);
      mongoac_database_drop(db, nullptr, nullptr, error);
      REQUIRE_MONGOAC_OK(error);

      // In this example, the final result must outlive the main loop.
      int sum = 0;

      // This struct is just to allow listing the operations in a sequential, non-nested order.
      struct operation_states {
         struct for_insert {
            mongoac_collection_t *coll = {};                // Non-owning.
            int *result = {};                               // Pointer to `sum`.
            std::shared_ptr<int> pending;                   // The latch against which to block scheduling of find ops.
            std::shared_ptr<std::array<bson_oid_t, 3>> ids; // The ObjectIDs of the documents to find.
            int v = {};                                     // The value with which to update "v".
         };

         static void
         on_insert(mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept
         {
            auto &for_insert = *static_cast<operation_states::for_insert *>(work.data.get());
            auto const reply_view = mongoac_future_get_bson(work.future, ctx.error);
            if (mongoac_error_code(ctx.error) != MONGOAC_ERROR_CODE_OK) {
               FAIL_CHECK("insert failed: " << owning_string(mongoac_error_message(ctx.error)).view());
               return;
            }

            // Validation.
            bson_t reply = {};
            if (!bson_init_static(&reply, reply_view.ptr, reply_view.len)) {
               FAIL_CHECK("insert failed: invalid BSON reply");
               return;
            }
            bson_iter_t iter = {};
            if (!bson_iter_init_find(&iter, &reply, "insertedId") || bson_iter_type(&iter) != BSON_TYPE_OID) {
               FAIL_CHECK("insert failed: missing or invalid 'insertedId' field");
               return;
            }
            auto const id = bson_iter_oid(&iter);

            ctx.queue(
               make_owning_ptr(
                  mongoac_collection_update_one_async(
                     for_insert.coll,
                     nullptr,
                     make_bson_view(make_owning_bson(BCON_NEW("_id", BCON_OID(id))).get()),
                     make_bson_view(make_owning_bson(BCON_NEW("$set", "{", "v", BCON_INT32(for_insert.v), "}")).get()),
                     nullptr,
                     ctx.error),
                  &mongoac_future_destroy),
               std::shared_ptr<operation_states::for_update>(new operation_states::for_update{
                  for_insert.coll,
                  for_insert.result,
                  std::move(for_insert.pending),
                  std::move(for_insert.ids),
                  *id,
                  static_cast<std::size_t>(for_insert.v) - 1u,
               }),
               on_update);
         }

         struct for_update {
            mongoac_collection_t *coll = {};                // Non-owning.
            int *result = {};                               // Pointer to `sum`.
            std::shared_ptr<int> pending;                   // The latch against which to block scheduling of find ops.
            std::shared_ptr<std::array<bson_oid_t, 3>> ids; // The ObjectIDs of the documents to find.
            bson_oid_t id = {};                             // The ObjectID of the document to update.
            std::size_t idx = {};                           // The ObjectID index for this operation.
         };

         static void
         on_update(mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept
         {
            auto const &[coll, result, pending, ids, id, idx] =
               *static_cast<operation_states::for_update *>(work.data.get());

            auto const reply_view = mongoac_future_get_bson(work.future, ctx.error);
            if (mongoac_error_code(ctx.error) != MONGOAC_ERROR_CODE_OK) {
               FAIL_CHECK("update failed: " << owning_string(mongoac_error_message(ctx.error)).view());
               return;
            }
            bson_t reply = {};
            if (!bson_init_static(&reply, reply_view.ptr, reply_view.len)) {
               FAIL_CHECK("update failed: invalid BSON reply");
               return;
            }

            // Validation.
            {
               bson_iter_t iter = {};
               auto const matched =
                  bson_iter_init_find(&iter, &reply, "matchedCount") ? bson_iter_as_int64(&iter) : std::int64_t{-1};
               auto const modified =
                  bson_iter_init_find(&iter, &reply, "modifiedCount") ? bson_iter_as_int64(&iter) : std::int64_t{-1};
               if (matched != 1 || modified != 1) {
                  FAIL_CHECK("update failed: matched " << matched << " documents, modified " << modified
                                                       << " documents");
                  return;
               }
            }

            *pending -= 1;
            (*ids)[idx] = id;

            // All update ops completed: queue find ops.
            if (*pending == 0) {
               // Note: C++20 is required to default-ref-capture structured bindings; use explicit ref-captures instead.
               auto const queue_find = [&, &coll = coll, &result = result](bson_oid_t id) {
                  ctx.queue(make_owning_ptr(mongoac_collection_find_one_async(
                                               coll,
                                               nullptr,
                                               make_bson_view(make_owning_bson(BCON_NEW("_id", BCON_OID(&id))).get()),
                                               nullptr,
                                               ctx.error),
                                            &mongoac_future_destroy),
                            std::shared_ptr<operation_states::for_find>(new operation_states::for_find{
                               result,
                            }),
                            on_find);
               };

               for (auto const &id : *ids) {
                  queue_find(id);
               }
            }
         }

         struct for_find {
            int *result = {}; // Pointer to `sum`.
         };

         static void
         on_find(mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept
         {
            auto const [result] = *static_cast<operation_states::for_find *>(work.data.get());

            auto const reply_view = mongoac_future_get_optional_bson(work.future, ctx.error);
            if (mongoac_error_code(ctx.error) != MONGOAC_ERROR_CODE_OK) {
               FAIL_CHECK("find failed: " << owning_string(mongoac_error_message(ctx.error)).view());
               return;
            }

            // Validation.
            if (!reply_view.ptr) {
               FAIL_CHECK("find failed: document not found");
               return;
            }
            bson_t reply = {};
            if (!bson_init_static(&reply, reply_view.ptr, reply_view.len)) {
               FAIL_CHECK("find failed: invalid BSON reply");
               return;
            }
            bson_iter_t iter = {};
            if (!bson_iter_init_find(&iter, &reply, "v") || bson_iter_type(&iter) != BSON_TYPE_INT32) {
               FAIL_CHECK("find failed: missing or invalid 'v' field");
               return;
            }

            *result += static_cast<int>(bson_iter_int32(&iter));
         }
      };

      {
         // This kind of manual setup would be unnecessary with C++20 Coroutines + `std::execution`, where the coroutine
         // stack may be conveniently passed along via `std::coroutine_handle<>`.
         auto const pending = std::make_shared<int>(3);
         auto const oids = std::make_shared<std::array<bson_oid_t, 3>>();

         auto const queue_insert = [&](int v) noexcept {
            ctx.queue(make_owning_ptr(mongoac_collection_insert_one_async(
                                         coll.get(),
                                         nullptr,
                                         make_bson_view(make_owning_bson(bson_from_json(R"({"v": 0})")).get()),
                                         nullptr,
                                         ctx.error),
                                      &mongoac_future_destroy),
                      std::shared_ptr<operation_states::for_insert>(
                         new operation_states::for_insert{coll, &sum, pending, oids, v}),
                      operation_states::on_insert);
         };

         queue_insert(1);
         queue_insert(2);
         queue_insert(3);
      }

      ctx.run(std::chrono::seconds(3));

      CHECK(sum == 6); // 1 + 2 + 3
   }

   // Run: test-libmongoac '[examples]' -p 'c:benchmarks'
   // Use `--skip-benchmarks` to skip these benchmarks (to validate correctness first).
   // Use `--benchmark-samples N` to increase sample count (default: 100).
   SECTION("benchmarks")
   {
      // Manually warm up libuv's thread pool.
      for (auto i = 0; i < 3; ++i) {
         std::atomic_bool warmup_done = false;
         uv_work_t warmup_work = {};
         warmup_work.data = &warmup_done;
         CHECK(uv_queue_work(
                  &loop,
                  &warmup_work,
                  +[](uv_work_t *) noexcept {},
                  +[](uv_work_t *work, int) noexcept {
                     static_cast<std::atomic_bool *>(work->data)->store(true, std::memory_order_relaxed);
                  }) == 0);
         while (!warmup_done.load(std::memory_order_relaxed)) {
            uv_run(&loop, UV_RUN_ONCE);
         }
      }

      // Manually warm up the mongoac connection pool.
      for (auto i = 0; i < 3; ++i) {
         (void)owning_bson(mongoac_database_run_command(db, nullptr, ping, nullptr, ctx.error));
         REQUIRE_MONGOAC_OK(ctx.error);
      }

      static constexpr auto on_ping = +[](mongoac_loop_ctx &ctx, mongoac_loop_ctx::work_type &work) noexcept {
         auto const reply = mongoac_future_get_bson(work.future, ctx.error);
         *static_cast<int *>(work.data.get()) += ping_ok(reply) ? 1 : 0;
      };

      for (auto n : {64, 128, 256, 512}) {
         DYNAMIC_SECTION("n=" << n)
         {
            BENCHMARK(std::to_string(n) + " sequential pings (sync)")
            {
               int count = 0;

               for (int i = 0; i < n; ++i) {
                  auto const reply = owning_bson(mongoac_database_run_command(db, nullptr, ping, nullptr, nullptr));
                  count += ping_ok(reply) ? 1 : 0;
               }

               return count;
            };

            BENCHMARK(std::to_string(n) + " concurrent pings (async)")
            {
               int count = 0;

               for (int i = 0; i < n; ++i) {
                  ctx.queue(make_owning_ptr(mongoac_database_run_command_async(db, nullptr, ping, nullptr, ctx.error),
                                            &mongoac_future_destroy),
                            std::shared_ptr<int>(&count, noop_deleter),
                            on_ping);
               }

               ctx.run(); // No timeout for benchmarks; validate correctness with `--skip-benchmarks` first.

               return count;
            };
         }
      }
   }
}

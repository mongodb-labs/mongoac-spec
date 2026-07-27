// Copyright 2009-present MongoDB, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0

#include <mongoac/database.h>

//

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>

#include <array>
#include <cstring>

namespace
{

template <typename T, typename D> class owning_ptr
{
 private:
   T *_ptr;
   D *_destroy;

 public:
   ~owning_ptr()
   {
      _destroy(_ptr);
   }

   owning_ptr(owning_ptr &&other) noexcept : _ptr{other._ptr}, _destroy{other._destroy}
   {
      other._ptr = nullptr;
      other._destroy = nullptr;
   }

   owning_ptr &
   operator=(owning_ptr &&other) noexcept
   {
      auto tmp = std::move(other);
      tmp.swap(*this);
      return *this;
   }

   owning_ptr(owning_ptr const &other) = delete;
   owning_ptr &
   operator=(owning_ptr const &other) = delete;

   explicit owning_ptr(T *ptr, D *destroy) : _ptr{ptr}, _destroy{destroy}
   {
      REQUIRE(ptr);
      REQUIRE(destroy);
   }

   void
   swap(owning_ptr &other) noexcept
   {
      std::swap(_ptr, other._ptr);
      std::swap(_destroy, other._destroy);
   }

   T *
   get() const
   {
      return _ptr;
   }

   /* explicit(false) */
   operator T *() const
   {
      return this->get();
   }
};

template <typename T, typename D>
owning_ptr<T, D>
make_owning_ptr(T *ptr, D *destroy)
{
   return owning_ptr<T, D>{ptr, destroy};
}

bool
bson_array_contains_string(const bson_t *array, const char *str)
{
   bson_iter_t iter = {};

   REQUIRE(array != nullptr);

   if (!bson_iter_init(&iter, array)) {
      return false;
   }

   while (bson_iter_next(&iter)) {
      const char *value = nullptr;

      if (!BSON_ITER_HOLDS_UTF8(&iter)) {
         continue;
      }

      value = bson_iter_utf8(&iter, nullptr);

      if (value && std::strcmp(value, str) == 0) {
         return true;
      }
   }

   return false;
}

} // namespace

TEST_CASE("create_collection_async", "[mongoac][database]")
{
   auto const client =
      make_owning_ptr(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db =
      make_owning_ptr(mongoac_client_get_database(client, "mongoac_database_create_collection_async", nullptr, nullptr),
                      &mongoac_database_destroy);

   // Clean test state.
   {
      mongoac_database_drop(db, nullptr, nullptr, nullptr);
      auto const names =
         make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);
      REQUIRE(bson_empty0(names.get()));
   }

   SECTION("async")
   {
      auto const a = make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, "a", nullptr, nullptr),
                                     &mongoac_future_destroy);
      auto const b = make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, "b", nullptr, nullptr),
                                     &mongoac_future_destroy);
      auto const c = make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, "c", nullptr, nullptr),
                                     &mongoac_future_destroy);

      {
         std::array<mongoac_future_t const *, 3u> futures = {a, b, c};
         mongoac_runtime_block_on_all(runtime, futures.data(), futures.size(), nullptr);
      }

      auto const names =
         make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

      CHECK(bson_array_contains_string(names, "a"));
      CHECK(bson_array_contains_string(names, "b"));
      CHECK(bson_array_contains_string(names, "c"));
   }

   SECTION("sync")
   {
      mongoac_database_create_collection(db, nullptr, "a", nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, "b", nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, "c", nullptr, nullptr);

      auto const names =
         make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

      CHECK(bson_array_contains_string(names, "a"));
      CHECK(bson_array_contains_string(names, "b"));
      CHECK(bson_array_contains_string(names, "c"));
   }
}

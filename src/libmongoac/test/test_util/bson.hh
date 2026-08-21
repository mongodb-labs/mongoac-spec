#pragma once

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/bson.h>

#include <cstddef>
#include <cstring>
#include <utility>

namespace mongoac
{
namespace test_util
{

bson_t *
bson_from_json(const char *json);

bool
bson_array_contains_string(const bson_t &array, const char *str);

inline mongoac_bson_view_t
make_bson_view(bson_t const *bson)
{
   return {bson_get_data(bson), bson->len};
}

class owning_bson
{
 private:
   mongoac_bson_t _owner;
   bson_t _bson;

 public:
   ~owning_bson()
   {
      mongoac_bson_destroy(_owner);
   }

   owning_bson(owning_bson &&other) noexcept : _owner{other._owner}, _bson{other._bson}
   {
      other._owner = mongoac_bson_t{nullptr, 0};
   }

   owning_bson(owning_bson const &other) = delete;
   owning_bson &
   operator=(owning_bson const &other) = delete;

   owning_bson &
   operator=(owning_bson &&other) noexcept
   {
      auto tmp = std::move(other);
      tmp.swap(*this);
      return *this;
   }

   explicit owning_bson(mongoac_bson_t d) : _owner{d}, _bson{}
   {
      if (_owner.ptr != nullptr) {
         bson_init_static(&_bson, _owner.ptr, _owner.len);
      } else {
         bson_init(&_bson);
      }
   }

   explicit
   operator bool() const
   {
      return _owner.ptr != nullptr;
   }

   mongoac_bson_t
   get() const
   {
      return _owner;
   }

   /* explicit(false) */
   operator mongoac_bson_view_t() const
   {
      return {_owner.ptr, _owner.len};
   }

   bson_t const &
   bson() const
   {
      return _bson;
   }

   /* explicit(false) */
   operator bson_t const &() const
   {
      return _bson;
   }

   bson_t const *
   bson_ptr() const
   {
      return &_bson;
   }

   /* explicit(false) */
   operator bson_t const *() const
   {
      return &_bson;
   }

   void const *
   data() const
   {
      return _owner.ptr;
   }

   std::size_t
   size() const
   {
      return _owner.len;
   }

   void
   swap(owning_bson &other) noexcept
   {
      std::swap(_owner, other._owner);
      std::swap(_bson, other._bson);
   }
};

} // namespace test_util
} // namespace mongoac

// Requires `owning_ptr<mongoac_error_t, ...> error;` to be in scope.
#define REQUIRE_MAKE_OWNING_BSON(expr)                                                     \
   [&, error = static_cast<::mongoac_error_t *>(error)] {                                  \
      auto ret = ::mongoac::test_util::owning_bson((expr));                                \
      CHECKED_IF(::mongoac_error_code(error) != 0)                                         \
      {                                                                                    \
         FAIL(::mongoac::test_util::owning_string(::mongoac_error_message(error)).view()); \
      }                                                                                    \
      return ret;                                                                          \
   }()

// Requires `owning_ptr<mongoac_error_t, ...> error;` to be in scope.
#define REQUIRE_BSON_VIEW(expr)                                                            \
   [&, error = static_cast<::mongoac_error_t *>(error)] {                                  \
      auto ret = (expr);                                                                   \
      CHECKED_IF(::mongoac_error_code(error) != 0)                                         \
      {                                                                                    \
         FAIL(::mongoac::test_util::owning_string(::mongoac_error_message(error)).view()); \
      }                                                                                    \
      bson_t bson;                                                                         \
      REQUIRE(::bson_init_static(&bson, ret.ptr, ret.len));                               \
      return bson;                                                                         \
   }()

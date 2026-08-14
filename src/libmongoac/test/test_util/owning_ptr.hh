#pragma once

#include <catch2/catch_test_macros.hpp>
#include <test_util/string.hh> // IWYU pragma: keep

#include <utility>

namespace mongoac
{
namespace test_util
{

template <typename T, typename D> class owning_ptr
{
 private:
   T *_ptr;
   D *_destroy;

 public:
   ~owning_ptr()
   {
      if (_ptr) {
         _destroy(_ptr);
      }
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
      REQUIRE(destroy);
   }

   explicit
   operator bool() const
   {
      return _ptr != nullptr;
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

   void
   swap(owning_ptr &other) noexcept
   {
      std::swap(_ptr, other._ptr);
      std::swap(_destroy, other._destroy);
   }
};

template <typename T, typename D>
owning_ptr<T, D>
make_owning_ptr(T *ptr, D *destroy)
{
   return owning_ptr<T, D>{ptr, destroy};
}

// Requires `owning_ptr<mongoac_error_t, ...> error;` to be in scope.
#define REQUIRE_MAKE_OWNING_PTR(ptr, destroy)                                              \
   [&, error = static_cast<::mongoac_error_t *>(error)] {                                  \
      auto ret = ::mongoac::test_util::make_owning_ptr((ptr), (destroy));                  \
      CHECKED_IF(::mongoac_error_code(error) != 0)                                         \
      {                                                                                    \
         FAIL(::mongoac::test_util::owning_string(::mongoac_error_message(error)).view()); \
      }                                                                                    \
      return ret;                                                                          \
   }()

} // namespace test_util
} // namespace mongoac

#pragma once

#include <catch2/catch_test_macros.hpp>

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

} // namespace test_util
} // namespace mongoac

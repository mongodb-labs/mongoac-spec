:man_page: bson_t

bson_t
======

BSON Document Abstraction

Synopsis
--------

.. code-block:: c

   #include <bson/bson.h>

   /* [Snipped: macros and helper declarations from original page] */

   typedef struct {
      uint32_t flags;       /* Internal flags for the bson_t. */
      uint32_t len;         /* Length of BSON data. */
      uint8_t padding[120]; /* Padding for stack allocation. */
   } bson_t;

Description
-----------

The :symbol:`bson_t` structure represents a BSON document. This structure manages the underlying BSON encoded buffer. For mutable documents, it can append new data to the document.

Performance Notes
-----------------

The :symbol:`bson_t` structure attempts to use an inline allocation within the structure to speed up performance of small documents. When this internal buffer has been exhausted, a heap allocated buffer will be dynamically allocated. Therefore, it is essential to call :symbol:`bson_destroy()` on allocated documents.

Duplicate Keys
--------------

The `BSON specification <https://bsonspec.org>`_ allows BSON documents to have duplicate keys. Documents are stored as an ordered list of key-value pairs. A :symbol:`bson_t` may contain duplicate keys. Applications should refrain from generating such documents,    because MongoDB server behavior is undefined when a BSON document contains duplicate keys.

.. Snipped.

Example
-------

.. code-block:: c

   static void
   create_on_heap (void)
   {
      bson_t *b = bson_new ();

      BSON_APPEND_INT32 (b, "foo", 123);
      BSON_APPEND_UTF8 (b, "bar", "foo");
      BSON_APPEND_DOUBLE (b, "baz", 1.23f);

      bson_destroy (b);
   }

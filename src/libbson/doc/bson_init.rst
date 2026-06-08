:man_page: bson_init

bson_init()
===========

Initialize a BSON document.

Synopsis
--------

.. code-block:: c

   #include <bson/bson.h>

   void
   bson_init (bson_t *b);

.. Snipped.

Description
-----------

   Initializes a :symbol:`bson_t` to an empty BSON document. The document is suitable for stack allocation and must be destroyed with :symbol:`bson_destroy()` when no longer needed.

.. Snipped.

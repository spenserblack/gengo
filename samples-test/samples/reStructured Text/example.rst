===============================
reStructuredText Syntax Guide
===============================

Basic Formatting
---------------
*italic text*
**bold text**
``inline code``

Headers
-------
Level 1
=======

Level 2
-------

Level 3
~~~~~~~

Lists
-----
* Bullet point
* Another point
    * Subpoint
    * Another subpoint

1. Numbered list
2. Second item
#. Auto-numbered item

Links
-----
`Link text <https://example.com>`_
External link_
.. _link: https://example.com

Code Blocks
----------
.. code-block:: python

        def hello():
                print("Hello World")

Tables
------
+------------+------------+
| Header 1   | Header 2   |
+============+============+
| Cell 1     | Cell 2     |
+------------+------------+

Simple Table:
===========  ===========
Header 1     Header 2
===========  ===========
Row 1        Value
Row 2        Value
===========  ===========

Images
------
.. image:: path/to/image.jpg
     :width: 100
     :alt: Alt text

Notes & Warnings
---------------
.. note::
     This is a note

.. warning::
     This is a warning

Directives
----------
.. contents:: Table of Contents
     :depth: 2

.. sidebar:: Sidebar Title
     :subtitle: Optional subtitle

     Sidebar content

.. topic:: Topic Title

     Topic content

Cross-References
---------------
.. _my-reference-label:

Section Title
------------
Reference to :ref:`my-reference-label`

Footnotes
---------
A footnote reference [1]_

.. [1] This is the footnote content

Comments
--------
.. This is a comment

Line Blocks
----------
| Line blocks preserve
| line breaks and
| leading whitespace

Definition Lists
---------------
term
        Definition of the term
another term
        Definition of another term
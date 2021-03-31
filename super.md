- `<super:content></super:content>` will be replaced with content of other files.
- `<super:wrap src="template.html"><!-- content --></super:content>` acts like include, but will place the content of it self at te position of `<super:content></super:content>` in the template. `<super:wrap> must always be a root element
- `<super:include src="REL_PATH/FILE.EXT"></super:include>` will include a file and evaluate it (html or markdown)
- The attribute `super:content="EXAMPLE_VAR"` replace the elements value with the value of `EXAMPLE_VAR`. Supersolid will error of the variable is not present
  ```html
  <!-- Assuming EXAMPLE_VAR is set to 'abc' -->
  <span super:content="EXAMPLE_VAR"></span>
  <!-- Will result in: -->
  <span>abc</span>
  ```
- The attribute `super:if="EXAMPLE_VAR"` will remove elements if the variable `EXAMPLE_VAR` is not present.
  This is especially useful when used with `<super:insert>`
- The attribute `super:for="i in ITER(ARGS)"` will duplicate the element for the amount of items in the specified iterator. The left hand side can be named arbitrarily and will contain the value of the iterator
    
  #### Available Iterators:
  - `FILES(GLOB)` Example: `f in FILES(src/*.md)`
  - `RANGE(START..END)` Example: `i in RANGE(0..10)` (END is exclusive)
  - `REGEX_SPLIT(VAR_NAME; /REGEX/)` Example: `line in REGEX_SPLIT(BIG_TEXT; /(?:\r?\n|\r)/)`
  
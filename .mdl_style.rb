all
exclude_rule 'MD029' # Ordered list item prefix
exclude_rule 'MD034' # Bare URL used
exclude_rule 'MD013' # Line length - too restrictive
exclude_rule 'MD022' # Headers surrounded by blank lines
exclude_rule 'MD031' # Fenced code blocks surrounded by blank lines
exclude_rule 'MD032' # Lists surrounded by blank lines
exclude_rule 'MD033' # Inline HTML - needed for logos/styling
exclude_rule 'MD041' # First line heading - README needs div for logo
exclude_rule 'MD026' # Trailing punctuation in headers - stylistic choice
exclude_rule 'MD024' # Multiple headers with the same content
exclude_rule 'MD040' # Fenced code blocks language - many examples without language
exclude_rule 'MD025' # Multiple top level headers - document structure
exclude_rule 'MD001' # Header levels increment - document structure
exclude_rule 'MD005' # Inconsistent list indentation - acceptable variation
exclude_rule 'MD055' # Table formatting - complex tables
exclude_rule 'MD057' # Table header separation - complex tables
exclude_rule 'MD007' # Unordered list indentation - too strict for nested lists

# Rule configurations
rule 'MD046', :style => :fenced

help:# 	help
### 	:
###MAKE	[COMMAND]
	@awk 'BEGIN {FS = ":.*?##"} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
-include Makefile
# vim: set noexpandtab:
# vim: set setfiletype make

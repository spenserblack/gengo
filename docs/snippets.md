# Snippets

These are snippets that might be fun for a user to try out.

## Build a history of language distribution

```shell
for rev in $(git log --reverse --format="%H"); do
	gengo git --rev "$rev"
	echo  # add a newline
done
```

init:
	pip install -U poetry
	poetry install

checkout:
	git submodule update --init --recursive --remote

build: checkout
	poetry run mkdocs build --strict --clean --verbose --site-dir public/book 

deploy: build
	cd public && git add book && git commit -m "Deploy book" && git push origin HEAD:master --force

serve:
	mkdocs serve
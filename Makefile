all:
	python setup.py build_ext --inplace

clean:
	python setup.py clean
	rm -rf *.so

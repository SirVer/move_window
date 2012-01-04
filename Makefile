%.cpp: %.pyx
	cython --cplus $< -o $@

all: _os_helper.cpp
	python setup.py build_ext --inplace

clean:
	python setup.py clean
	rm -rf *.so _os_helper.c _os_helper.cpp _os_helper.cc

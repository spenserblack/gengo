function mkcd --wraps=mkdir --description 'Make a directory and cd into it' --argument directory
mkdir -p $directory
cd $directory
end

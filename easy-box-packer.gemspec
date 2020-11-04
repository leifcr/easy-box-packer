# coding: utf-8

Gem::Specification.new do |s|
  s.name         = 'easy-box-packer'
  s.version      = '0.0.12'
  s.author       = 'Aloha Chen'
  s.email        = 'y.alohac@gmail.com'
  s.homepage     = 'https://github.com/leifcr/easy-box-packer'
  s.license      = 'MIT'
  s.summary      = '3D bin-packing with weight limit using first-fit decreasing algorithm and empty maximal spaces'
  s.files        = Dir['LICENSE.txt', 'README.md', 'easy-box-packer.rb']
  s.require_path = '.'
  s.add_dependency 'rutie', '~> 0.0.4'
  s.add_development_dependency 'guard'
  s.add_development_dependency 'guard-rspec'
  s.add_development_dependency 'pry'
  s.add_development_dependency 'rspec', '~> 3.1', '>= 3.1.0'
  s.extensions = %w[ext/Rakefile]
end

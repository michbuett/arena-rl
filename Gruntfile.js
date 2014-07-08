/* global module */
module.exports = function(grunt) {
    'use strict';

    grunt.initConfig({
        pkg: grunt.file.readJSON('package.json'),

        jsonlint: {
            all: {
                src: [ 'data/**/*.json' ]
            }
        },

        jshint: {
            files: ['Gruntfile.js'],
            options: {
                jshintrc: '.jshintrc'
            }
        },

        sass: {
            dev: {
                options: {
                    sourcemap: true,
                    style: 'expanded',
                    lineNumbers: true,
                    debugInfo: true,
                },

                files: {
                    'css/arena.css': 'scss/arena.scss',
                }
            },

            release: {
                options: {
                    style: 'compressed'
                },
                files: {
                    'css/arena.css': 'scss/arena.scss',
                }
            }
        },

        watch: {
            files: ['<%= jshint.files %>'],
            tasks: ['jshint']
        }
    });

    grunt.loadNpmTasks('grunt-contrib-jshint');
    grunt.loadNpmTasks('grunt-contrib-watch');
    grunt.loadNpmTasks('grunt-jsonlint');
    grunt.loadNpmTasks('grunt-contrib-sass');

    grunt.registerTask('test', ['jsonlint', 'jshint']);
};

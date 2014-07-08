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

        watch: {
            files: ['<%= jshint.files %>'],
            tasks: ['jshint']
        }
    });

    grunt.loadNpmTasks('grunt-contrib-jshint');
    grunt.loadNpmTasks('grunt-contrib-watch');
    grunt.loadNpmTasks('grunt-jsonlint');

    grunt.registerTask('test', ['jsonlint', 'jshint']);
};

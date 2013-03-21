(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.Player',
        extend: 'alchemy.core.Oculus',
        overrides: {
            col: undefined,
            row: undefined,

            init: function () {
                var pos = this.map.getStartPos();
                this.col = pos[0];
                this.row = pos[1];
            },

            update: function () {
            },

            moveTo: function () {
            }
        }
    });
}());


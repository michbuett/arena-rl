(function () {
    'use strict';

    var alchemy = require('./alchemy.js');
    alchemy.formula.add({
        name: 'arena.EntityView',
        extend: 'arena.View',
        overrides: {
            width: 32,
            height: 32,

            template: [
                '<div id="<$=data.id$>"',
                ' class="entity <$=data.cls$>"',
                ' style="left:<$=data.x$>px; top:<$=data.y$>px;"',
                '</div>'
            ].join(''),

            getData: function hocuspocus(_super) {
                return function () {
                    return alchemy.mix(_super.call(this), {
                        x: this.entity.col * this.width,
                        y: this.entity.row * this.height
                    });
                };
            }
        }
    });
}());


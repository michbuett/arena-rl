(function () {
    /**
     * Description
     *
     * @class Alchemy.v.Factory
     * @extends Alchemy.BaseType
     */
    var Factory = {
        mapping: undefined,


        /**
         * Registers the prototype to a view type
         *
         * @param {String} vtype
         *      the view type identifyer
         *
         * @param {Object} proto
         *      the prototype for new view instances
         */
        registerView: function (vtype, proto) {
            this.setMapping(vtype, {
                prototype: proto
            });
        },

        /**
         * Registers a controller to a view type
         *
         * @param {String} vtype
         *      The view type identifier
         *
         * @param {Object} controller
         *      The controller instance
         */
        registerController: function (vtype, controller) {
            var mapping = this.getMapping(vtype) || {};
            // add instance to the list of controllers
            mapping.controllers = mapping.controllers || [];
            mapping.controllers.push(controller);
            this.setMapping(vtype, mapping);
        },

        /**
         * Creates a new view instance
         *
         * @param {Object} cfg
         *      The view configuration; The property <code>vtype</code> is required
         *      to identify the views prototype; if no such prototype can be found
         *      or if the property is missing, then an exception will be thrown
         *
         * @return {Object}
         *      the new view instance
         */
        produce: function (cfg) {
            var vtype = cfg.vtype.toLowerCase(),
                mapping = this.getMapping(vtype),
                prototype = mapping && mapping.prototype,
                view, i, l;

            if (!prototype) {
                throw 'Unknown type: ' + vtype;
            }
            // create view instance
            delete cfg.vtype;
            view =  prototype.create(cfg);
            // bind registered controller to view
            if (mapping.controllers) {
                for (i = 0, l = mapping.controllers.length; i < l; i++) {
                    mapping.controllers[i].control(vtype, view);
                }
            }
            return view;
        },

        /**
         * @private
         */
        getMapping: function (vtype) {
            return this.mapping && this.mapping[vtype.toLowerCase()];
        },

        /**
         * @private
         */
        setMapping: function (vtype, mapping) {
            // enrich/override existing mappings
            mapping = Alchemy.mix(this.getMapping(vtype) || {}, mapping);
            // store data
            this.mapping = this.mapping || {};
            this.mapping[vtype.toLowerCase()] = mapping;
        }
    };

    Alchemy.ns('Alchemy.v');
    Alchemy.v.Factory = Alchemy.brew({
        name: 'Factory',
        ns: 'Alchemy.v',
        extend: Alchemy.BaseType
    }, Factory);
})();
